use content_managment_datamodel::print::{PrintDefinition,PrintCompiled};
use content_managment_datamodel::datamodel::{Resources,GeneratedDataDesc,ImageMetadata};
use reqwest::{StatusCode,Client};
use std::fmt;
use log::info;
use std::collections::HashMap;

#[derive(Eq,PartialEq,Debug,Clone)]
pub struct State
{
    resources: content_managment_datamodel::datamodel::Resources,
    resources_etag: Option<String>,
    prints: PrintDefinition<PrintCompiled>,
    prints_etag: Option<String>,
    lookup_cache: HashMap<String,String>
}


impl State 
{
    pub fn new() -> State
    {
        State {
               resources: Default::default(),
               resources_etag: None,
               prints: PrintDefinition {
                    variants: HashMap::new(),
                    prints: HashMap::new()
               },
               prints_etag: None,
               lookup_cache: HashMap::new()
            }
    }

    pub fn resources(&self) -> &Resources 
    {
        &self.resources
    }
    
    pub fn prints(&self) -> &PrintDefinition<PrintCompiled> 
    {
        &self.prints
    }

    pub fn lookup_id(&self,name: &str) -> Option<&String>
    {
        self.lookup_cache.get(name)
    }
    fn generate_cache(&mut self) 
    {

        self.lookup_cache = self.prints.prints
        .iter()
        .map(|(_k,v)| v)
        .flatten()
        .map(|print|  {
            let res = self.resources().resources.get(&print.id).unwrap();
            let name = &res.as_data::<ImageMetadata>().name;
            ( String::from(name), print.id.clone())
        } )
        .collect();

        info!("{:#?}",self.lookup_cache);

    }

}


pub struct FetchError
{
    error: String
}

impl FetchError
{
    fn error(err: String) -> FetchError
    {
        FetchError {
            error: err
        }
    }
}

impl fmt::Display for FetchError 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.error)
    }
}

impl std::convert::From<reqwest::Error> for FetchError 
{
    fn from(t :reqwest::Error) -> FetchError
    {
        FetchError
        {
            error: format!("{}",t)
        }
    }
}

impl std::convert::From<serde_yaml::Error> for FetchError 
{
    fn from(t : serde_yaml::Error) -> FetchError
    {
        FetchError
        {
            error: format!("{}",t)
        }
    }
}
impl std::convert::From<serde_json::Error> for FetchError 
{
    fn from(t : serde_json::Error) -> FetchError
    {
        FetchError
        {
            error: format!("{}",t)
        }
    }
}


async fn fetch_if_modified(client: &Client, path: &String, e_tag : &Option<String> ) -> Result<Option<(String,Option<String>)>,FetchError>
{
    info!("Fetching {}",path);
    let  req =  client.get(path);
    let  req = 
    {
        if let Some(e_tag) = e_tag
        {
            req.header("If-None-Match",e_tag)
        }
        else 
        {
            req
        }

    };
    let resp = req.send().await?;
    match resp.status()
    {
        StatusCode::OK => 
        {
            let etag = resp.headers().get("etag").map(|x| String::from(x.to_str().unwrap()));
            info!("Fetched {} etag {:#?}",path,etag);
            Ok(Some( (resp.text().await?,  etag ) ) )
        },
        StatusCode::NOT_MODIFIED => 
        {
            info!("Not modifed {}",path);
            Ok(None)
        },
        code => 
        {
            Err(FetchError::error(format!{"Server returned {} for path \"{}\"",code,path}))
        }
    }

}


pub async fn fetch(path: &str, state: &State ) -> Result<State,FetchError>
{
    let client = reqwest::Client::new();
    let manifest_path = format!("{}/manifest.yaml",path);
    match fetch_if_modified(&client, &manifest_path, &state.resources_etag).await?
    {
        Some((payload,etag)) => 
        {
           let mut new_state = State::new();
           new_state.resources = serde_yaml::from_str(&payload)?;
           new_state.resources_etag = etag;

           if let Some(prints) =  new_state.resources.find_data( |x : &GeneratedDataDesc| x.name == "prints" )
           {
               let etag = {
                   if let Some(old_prints) =  state.resources.find_data( |x : &GeneratedDataDesc| x.name == "prints" )
                   {
                        if old_prints == prints 
                        {
                            new_state.prints = state.prints.clone();
                            state.prints_etag.clone()

                        }
                        else {
                            None
                        }
                   }
                   else 
                   {
                       None
                   }
               }; 
               
               let prints_path = format!("{}/{}",path,prints.path().display());
               if let Some((payload,etag)) =  fetch_if_modified(&client,&prints_path , &etag).await?
               {
                    new_state.prints = serde_json::from_str(&payload)?;
                    new_state.generate_cache();
                    new_state.prints_etag = etag;
               }

           }
           else
           {
                info!("Prints not found in resources!");
           }
           Ok(new_state)
        },
        None => 
        {
            Ok(state.clone())

        }
    }
   
}