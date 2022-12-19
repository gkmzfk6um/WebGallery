mod fetch;
mod util;
use log::info;

use actix_web::{get, post, web, App,middleware::Logger, HttpResponse, HttpServer, Responder};
use content_managment_datamodel::print::PrintCompiled;
use std::sync::RwLock;
use std::env;

struct AppState 
{
    state: RwLock<fetch::State>
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


fn print_response(print: &PrintCompiled, state : &fetch::State) -> actix_web::HttpResponse
{
   let image = state.resources().resources.get(&print.id).unwrap().clone();
   let thumbnails = util::get_thumbnails(&image,state.resources());
   let resp =
       &content_managment_datamodel::api::PrintApi {

           variants: util::get_variants(&print,state.prints()),
           description: print.description.clone(),
           brief: print.brief.clone(),
           image: image,
           thumbnails: thumbnails
       };
   HttpResponse::Ok().json(resp)

}

#[get("/print/name/{print_name}")]
async fn print_by_name(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let print_name = path.into_inner();
    match data.state.read()
    {
        Ok(state) => 
        {
            match (*state).lookup_id(&print_name) 
            {
                Some(print_id) => 
                {
                    let print = util::find_print(&print_id, state.prints()).unwrap();
                    print_response(&print, &*state)
                }
                None => {
                    HttpResponse::NotFound().body("")
                }
            }
        },
        Err(_) => {
            HttpResponse::InternalServerError().body("Could not acquire context!")
        }

    }
}

#[get("/print/id/{print_id}")]
async fn print_by_id(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let print_id = path.into_inner();
    match data.state.read()
    {
        Ok(state) => 
        {
            match util::find_print(&print_id, state.prints())
            {
                Some(print) => 
                {
                    print_response(&print, &*state)
                }
                None => {
                    HttpResponse::NotFound().body("")
                }
            }
        },
        Err(_) => {
            HttpResponse::InternalServerError().body("Could not acquire context!")
        }

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
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}