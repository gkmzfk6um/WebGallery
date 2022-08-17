use serde_json::json;
use reqwest;
use serde::{Serialize,Deserialize};
use std::hash::{Hash,Hasher};
use tokio::{task};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::vec::Vec;
use std::fmt;
use indicatif::ProgressBar;
use colored::Colorize;
use std::env;

use crate::datamodel::{Resource,ImageMetadata,ResourceData,SiteDataConfig};
use crate::config::ResourceFilter;
use crate::datamodel::data::image::ImageMetadataError;
use crate::datamodel::data::sitedata::SiteDataConfigError;

const API_KEY_ENV_NAME : &str = "CONTENT_MANAGMENT_DROPBOX_API_KEY";
lazy_static! {
    static ref API_KEY : String =  env::var(API_KEY_ENV_NAME).expect(format!("Environment variable {} must be set to use dropbox API",API_KEY_ENV_NAME).as_str());
}


 #[derive(Serialize, Deserialize,Eq,PartialEq,Debug)]
 pub struct DropboxManifestEntry
 {
     pub name: String,
     pub id: String,
     pub path_display: String,
     pub content_hash: String
 }

impl Hash for DropboxManifestEntry
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

 #[derive(Serialize, Deserialize,Eq,PartialEq,Default)]
 pub struct DropboxManifest
 {
     pub files: std::collections::HashMap<String,DropboxManifestEntry> 
 }


 #[derive(Deserialize)]
struct DropboxListFolderMeta {
    id : String,
    #[serde(rename = ".tag")]
    tag : String,
    name: String,
    path_display: String,
    //client_modified: DateTime<Utc>,
    //server_modified: DateTime<Utc>,
    content_hash: Option<String>,
    //rev: Option<String>,
    //size: u64,

}


#[derive(Debug)]
pub enum DropboxError
{
    Str(String),
    Reqwest(reqwest::Error),
    Tokio(tokio::task::JoinError),
    MetadataError(ImageMetadataError),
    ResourceError(SiteDataConfigError)
}

impl fmt::Display for DropboxError 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self
        {
            DropboxError::Str(x) => write!(f, "{}", x),
            DropboxError::Reqwest(x) => write!(f, "{}", x),
            DropboxError::Tokio(x) => write!(f, "{}", x),
            DropboxError::MetadataError(x) => write!(f, "{:#?}", x),
            DropboxError::ResourceError(x) => write!(f, "{:#?}", x),
        }
    }
}

impl DropboxError {
    fn new(s : String) -> DropboxError {
        DropboxError::Str(s)
    }
}


impl From<reqwest::Error> for DropboxError {
    fn from(error: reqwest::Error) -> Self {
        DropboxError::Reqwest(error)
    }    
}

impl From<tokio::task::JoinError> for DropboxError {
    fn from(error: tokio::task::JoinError) -> Self {
        DropboxError::Tokio(error)
    }    
}

impl From<String> for DropboxError {
    fn from(error: String) -> Self {
        DropboxError::Str(error)
    }    
}
impl From<SiteDataConfigError> for DropboxError {
    fn from(error: SiteDataConfigError) -> Self {
        DropboxError::ResourceError(error)
    }    
}
impl From<ImageMetadataError> for DropboxError {
    fn from(error: ImageMetadataError) -> Self {
        DropboxError::MetadataError(error)
    }    
}


type ManifestResult=  Result<DropboxManifest,DropboxError>;

async fn send_continue_list_request(client : &reqwest::Client, has_more: bool,  cursor: String ) -> Result<Option<reqwest::Response>,reqwest::Error>
{
    if has_more
    {
        #[derive(Serialize)]
        struct ListFolderContinueRequest {
            cursor: String
        }

        client.post("https://api.dropboxapi.com/2/files/list_folder/continue")
              .header(reqwest::header::AUTHORIZATION, format!("Bearer {}",API_KEY.as_str() ))
              .header(reqwest::header::CONTENT_TYPE, "json" )
              .json(&ListFolderContinueRequest {
                    cursor
                })
               .send()
               .await.and_then(|x| Ok(Some(x)) )
    }
    else 
    {
        Ok(None)
    }
    
}

async fn send_first_list_request(client : &reqwest::Client) -> Result<reqwest::Response,reqwest::Error>
{
    #[derive(Serialize)]
    struct ListFolderRequest {
        path: String,
        recursive: bool,
        include_media_info: bool,
        include_deleted: bool,
        limit: u32
    }
    
    client.post("https://api.dropboxapi.com/2/files/list_folder")
          .header(reqwest::header::AUTHORIZATION, format!("Bearer {}",API_KEY.as_str()))
          .header(reqwest::header::CONTENT_TYPE,"json" )
          .json(&ListFolderRequest {
                path: String::from(crate::dropbox::DROPBOX_FOLDER),
                recursive: true,
                include_media_info: false,
                include_deleted: false,
                limit: 200
            })
            .send()
            .await

}


pub async fn get_manifest(client : &reqwest::Client) -> ManifestResult {

    let mut manifest =  DropboxManifest { files: std::collections::HashMap::new() };
    

    let mut dropbox_response = send_first_list_request(&client).await?;

    loop 
    {
        if dropbox_response.status() != reqwest::StatusCode::OK
        {
            return Err( DropboxError::from(format!("Dropbox responend with non OK status\n {:#?}",dropbox_response)));
        }

        #[derive(Deserialize)]
        struct DropboxListFolderResponse {
            cursor: String,
            entries: std::vec::Vec<DropboxListFolderMeta>,
            has_more: bool
        }
    
        let mut json_response = dropbox_response.json::<DropboxListFolderResponse>().await?;

        let new_client = client.clone();
        let next_request = task::spawn( async move {
            send_continue_list_request(&new_client,json_response.has_more, json_response.cursor).await
         }
        );

        while let Some(e) =  json_response.entries.pop()
        {
            let id = &e.id[3..];
            if e.tag == "file" {
                manifest.files.insert(String::from(id),DropboxManifestEntry { 
                    name: e.name,
                    id: String::from(id),
                    path_display: e.path_display,
                    content_hash: e.content_hash.unwrap()
                });
            }
        }

        match next_request.await??
        {
            Some(req) => dropbox_response = req,
            None => return Ok(manifest)

        };

    }
}


async fn dropbox_fetch_resource(client : &reqwest::Client, id : &str, path : &str) -> Result<(),DropboxError>
{

    let mut file = match File::create(&path).await
    {
        Ok(f) => f,
        Err(_) => return Err(DropboxError::new(format!("Failed to open {}",&path)))
    };
    


    let mut resp = client.post("https://content.dropboxapi.com/2/files/download")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}",API_KEY.as_str() ))
        .header("Dropbox-API-Arg", json!({"path": format!("id:{}",id) }).to_string() )
         .send()
         .await?;
    
    
    
    match resp.status()
    {
        reqwest::StatusCode::OK => {
            while let Some(chunk) = resp.chunk().await? {
                match file.write_all(&chunk).await
                {
                    Ok(_) => (),
                    Err(_) => return Err(DropboxError::new(String::from("I/O ERROR")))
                };
            }
            Ok(())
        },
        s => Err(DropboxError::new(format!("Failed to fetch resource id {}. Endpoint returned code {}!",id,s)))
    }
    

}

fn allocate_resource(name :&str , path : &str, filter: &ResourceFilter) -> Result<ResourceData,DropboxError>
{
    match filter
    {
        ResourceFilter::Data  => 
        {
            SiteDataConfig::new(&name,&path ).map( |x| ResourceData::Sitedata(x) ).map_err(|x| DropboxError::from(x))
        },
        ResourceFilter::Images => 
        {

            ImageMetadata::new(&name,&path).map( |x| ResourceData::Image(x) ).map_err(|x| DropboxError::from(x))

        }
    }
}

async fn join_handles(bar :&mut ProgressBar,errors : &mut Vec<DropboxError>, resources : &mut Vec<Resource> ,handles : Vec<tokio::task::JoinHandle<Result<Resource,DropboxError>>> )
{
    for handle in handles
    {
        match handle.await.expect("Joining data add task failed!")
        {
            Ok(new_resource) => {
                resources.push(new_resource);
            },
            Err(e) => {
                errors.push(e);
            }
        };
        bar.inc(1);
    }
}

pub async fn fetch_resources(client: &reqwest::Client, manifest : Vec<DropboxManifestEntry>, filter: &ResourceFilter) -> Vec<Resource>
{
    let len = manifest.len();
    let mut resources = Vec::with_capacity(len);
    if len == 0
    {
        return resources;
    }
    println!("Fetching {:#?}...",filter);
    const CHUNK_SIZE : usize = 4;
    let mut bar = ProgressBar::new(manifest.len().try_into().unwrap());
    bar.tick();

    let mut handles = Vec::with_capacity(CHUNK_SIZE);
    let mut errors = Vec::new();
    for manifest_entry in manifest
    {
        let new_client = client.clone();
        let filter : ResourceFilter = *filter;
        handles.push(tokio::task::spawn(
            async move {
                let path = &crate::config::resource_path(&manifest_entry.name,&filter);
                match dropbox_fetch_resource(&new_client,&manifest_entry.id,path).await
                {
                    Ok(_) => 
                    {
                        
                        match allocate_resource(&manifest_entry.name,&path,&filter)
                        {
                            Ok(resource_data) =>
                            {
                                let resource = Resource::new(
                                    String::from(path),
                                    resource_data,
                                    &manifest_entry.id,
                                    &manifest_entry.content_hash,
                                    crate::datamodel::ResourceProvider::Dropbox,
                                );
                                resource.write_resource();
                                Ok(resource)
                            },
                            Err(e) => Err(e) 
                        }

                    },
                    Err(e) => Err(e)
                }
            }
        ));
        
        if handles.len() == CHUNK_SIZE
        {
            join_handles(&mut bar,&mut errors,&mut resources,handles.drain(..).collect() ).await;
        }
    }

    join_handles(&mut bar,&mut errors, &mut resources,handles.drain(..).collect() ).await;
    bar.finish();

    if errors.len() > 0
    {
        println!("{} operations failed",format!("{}/{}",errors.len(),len).red());
        for err in errors
        {
            println!("{}",format!("{}",err).red());
        }
    }

    if resources.len() > 0
    {
        println!("{} fetched successfully",format!("{}/{}",resources.len(),len).green());
    }


    resources
}