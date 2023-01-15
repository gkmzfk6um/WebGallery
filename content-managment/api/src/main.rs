mod fetch;
mod util;
mod store;
#[macro_use]
extern crate lazy_static;
use log::{info,warn};

use actix_web::{get, post, web, App,middleware::Logger, HttpResponse, HttpServer, Responder};
use actix_web::http::header::ContentType;
use content_managment_datamodel::print::PrintCompiled;
use std::sync::{RwLock,RwLockReadGuard,PoisonError};
use std::env;
use content_managment_datamodel::api::InfoResponse;
use content_managment_datamodel::api::InfoItem;
use content_managment_datamodel::api::InfoVariant;
use content_managment_datamodel::api::CheckoutCart;
use content_managment_datamodel::datamodel::ImageMetadata;

struct AppState 
{
    state: RwLock<fetch::State>
}

lazy_static! 
{
    pub static ref API_VERSION : u32 =  content_managment_datamodel::DATAMODEL_MAJOR_VERSION.parse::<u32>().unwrap();
}
#[derive(Debug)]
struct AppResponseError 
{
    message: String,
    status_code: actix_web::http::StatusCode
}

impl std::fmt::Display for AppResponseError 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",&self.message)
    }
}

impl actix_web::ResponseError for AppResponseError
{
    fn status_code(&self) -> actix_web::http::StatusCode
    {
        self.status_code
    }
}

impl std::convert::From<serde_json::Error> for AppResponseError 
{
    fn from( e : serde_json::Error) -> AppResponseError
    {
        warn!("AppResponseError {:#?}",e);
        AppResponseError 
        {
            message: String::from("JSON serialization/deserialization failure"),
            status_code: actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}


type RwPosionError<'a> = PoisonError<RwLockReadGuard<'a, fetch::State>>;
impl std::convert::From<RwPosionError<'_>> for AppResponseError 
{
    fn from( e : RwPosionError<'_> ) -> AppResponseError
    {
        warn!("AppResponseError {:#?}",e);
        AppResponseError 
        {
            message: String::from("Internal SYNC failed"),
            status_code: actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn print_response(print: &PrintCompiled, state : &fetch::State) -> content_managment_datamodel::api::PrintApi
{
   let image = state.resources().resources.get(&print.id).unwrap().clone();
   let thumbnails = util::get_thumbnails(&image,state.resources());
    content_managment_datamodel::api::PrintApi {
        variants: util::get_variants(&print,state.prints()),
        description: print.description.clone(),
        brief: print.brief.clone(),
        image: image,
        thumbnails: thumbnails
    }

}

#[get("/ok")]
async fn is_ok() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/update")]
async fn update(data: web::Data<AppState>) -> impl Responder {
   let resp =  match data.state.try_write()
    {
        Ok(mut state) => {
            let url = env::var("DATA_URL");
            if let Ok(url) = url 
            {
                match fetch::fetch(&url, &*state).await
                {
                    Ok(new_state) => {
                        *state = new_state;
                        info!("update - resources:{} prints:{} ", (*state).resources().resources.len(),(*state).prints().prints.len() );
                        HttpResponse::Ok().finish()
                    },
                    Err(e) => {
                        HttpResponse::InternalServerError().body(format!("{}",e))
                    }

                }
            }
            else {
                HttpResponse::InternalServerError().body("Path to data not defined")
            }
        },
        Err(_) => {
            HttpResponse::Conflict().finish()
        }
    };


    resp
}



#[get("/print/name/{print_name}")]
async fn print_by_name(path: web::Path<String>, data: web::Data<AppState>) -> Result<HttpResponse,AppResponseError> {
    let print_name = path.into_inner();
    let state = data.state.read()?;
    match (*state).lookup_id(&print_name) 
    {
        Some(print_id) => 
        {
            let print = util::find_print(&print_id, state.prints()).unwrap();
            Ok(HttpResponse::Ok().json(print_response(&print, &*state)))
        }
        None => {
            Ok(HttpResponse::NotFound().body(""))
        }
    }

}

#[get("/print/id/{print_id}")]
async fn print_by_id(path: web::Path<String>, data: web::Data<AppState>) ->Result<HttpResponse,AppResponseError> {
    let print_id = path.into_inner();
    let state = data.state.read()?;
    match util::find_print(&print_id, state.prints())
    {
        Some(print) => 
        {
            Ok(HttpResponse::Ok().json(print_response(&print, &*state)))
        }
        None => {
            Ok(HttpResponse::NotFound().body(""))
        }
    }
}



#[post("/print/info")]
async fn print_info(body: String,  data: web::Data<AppState>) -> Result<HttpResponse,AppResponseError> {
    type InfoRequest = Vec<String>;
    let req_ids : InfoRequest = serde_json::from_str(&body)?;
    
    let mut resp = InfoResponse {
        failed: Vec::new(),
        success: std::collections::HashMap::new()
    };


    let state = data.state.read()?;

    for id in req_ids
    {
        match util::find_print(&id, state.prints())
        {
            Some(print) => 
            {
                let mut print = print_response(&print, &*state);
                let variants: std::collections::HashMap<String,InfoVariant> =  print.variants.drain().map( |(k,v)|  (k, InfoVariant {
                            width: v.width,
                            height: v.height,
                            price: v.price.value
                        }  )).collect();
                let name = &print.image.as_data::<ImageMetadata>().name;

                resp.success.insert(id,
                    InfoItem 
                    {
                        name: name.to_string(),
                        variants

                    }
                );
            }
            None => {
                resp.failed.push(id);
            }
        }
    }

    Ok(HttpResponse::Ok().json(resp))
}

#[post("/store/checkout")]
async fn checkout(body: String,  data: web::Data<AppState>) -> Result<HttpResponse,AppResponseError> {
    let cart : CheckoutCart = serde_json::from_str(&body)?;
    
    info!("User checkout initiated");
    if cart.version != *API_VERSION
    {
        return Ok(HttpResponse::NotAcceptable().insert_header(ContentType::plaintext()).body(
                format!("INVALID API VERSION\nSupported version: {}",*API_VERSION)
            )
        )
    }
    
    let state = data.state.read()?;
    match store::validate_cart(&state,cart)
    {
        None =>  Ok(HttpResponse::BadRequest().insert_header(ContentType::plaintext()).body("Cart items could not be validated")),
        Some(cart) => Ok(store::checkout_cart(&cart).await)
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let state = web::Data::new(AppState {
        state: RwLock::new(fetch::State::new())
    });
    env_logger::init();

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .app_data(state.clone())
            .service(is_ok)
            .service(update)
            .service(print_by_name)
            .service(print_by_id)
            .service(print_info)
            .service(checkout)
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}