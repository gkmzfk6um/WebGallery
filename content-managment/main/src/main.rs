mod datamodel;
mod dropbox;
mod config;
mod generate;
mod cleanup;
mod clone;
#[macro_use]
extern crate lazy_static;

use content_managment_datamodel::datamodel::Resources;
use crate::datamodel::resource_file_manager::ResourcesFileManager;
use crate::dropbox::populate_using_dropbox;
use clap::{Parser,ValueEnum};
use std::fs;
use std::path::Path;
use regex::Regex;


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum,Debug)]
pub enum CleanTargets 
{
    Thumbnails,
    Data,
    All
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum,Debug)]
pub enum ManifestOptions 
{
    Yaml,
    Json
}

/// Web gallery file managment and content generator
#[derive(Debug,Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {

   /// Clean specified categories of files
   #[clap(arg_enum,short, long, value_parser )]
   clean: Option<CleanTargets>,

   
   #[clap(short, long, value_parser)]
   clone_url: Option<String>,

   
   #[clap(short, long, value_parser)]
   print_id: Option<String>,
   
   
   #[clap(long, value_parser)]
   print_name: Option<String>,
   
   // Generate derived resources
   #[clap(short, long, value_parser,default_value="false")]
   generate: bool,
   
   // Create resource directory tree if it doesn't exist
   #[clap(long, value_parser,default_value="false")]
   create_dir: bool,
   
   // Create resource directory tree if it doesn't exist
   #[clap(long, value_parser,default_value="false")]
   sync_dropbox: bool,

   // print manifest in specified format to std::out
   #[clap(arg_enum,short, long, value_parser)]
   manifest: Option<ManifestOptions>,


   // Set the folder to operate on
   #[clap(short, long, value_parser,default_value="./")]
   root: std::path::PathBuf

}

lazy_static! {
    pub static ref ARGS : Cli = {
        let mut args = Cli::parse();
        args.root =fs::canonicalize(args.root).expect("Root folder must exist!");
        args
    };

    pub static ref WEBSITE_NAME : String = {
        std::env::var("GALLERY_URL").expect("Environment variable GALLERY_URL must be set to indicate the url of the website")
    };
}

fn create_resource_folder()
{
    if ARGS.create_dir
    {
        let resource_dir = ARGS.root.join(&Path::new("resources"));
        fs::create_dir_all(&resource_dir.join(&Path::new("data")      )).expect("Could not create data folder");
        fs::create_dir_all(&resource_dir.join(&Path::new("resource-images"))).expect("Could not create resource-images folder");
        fs::create_dir_all(&resource_dir.join(&Path::new("images")    )).expect("Could not create images folder");
        fs::create_dir_all(&resource_dir.join(&Path::new("meta")      )).expect("Could not create meta folder");
        fs::create_dir_all(&resource_dir.join(&Path::new("thumbnails"))).expect("Could not create thumbnails folder");
    }
}



#[tokio::main(worker_threads = 6)]
async fn main() {

    create_resource_folder();

    let mut res = {
        let mut stored_resources = Resources::read_resources();
        cleanup::cleanup(&mut stored_resources);
        if let Some(url) = &ARGS.clone_url 
        {
            match clone::clone_node(&url, stored_resources).await
            {
                Err(e) => {
                    println!("{}",e);
                    std::process::exit(10)
                }
                Ok(resources) => {
                    resources
                }
            }
        }
        else if ARGS.sync_dropbox {
            populate_using_dropbox(stored_resources).await
        }
        else {
            stored_resources
        }
    };
    
    if let Some(op) = ARGS.clean  {
            match op {
                CleanTargets::Thumbnails => {cleanup::remove_thumbnails(&mut res); },
                CleanTargets::Data => {cleanup::remove_data(&mut res); },
                CleanTargets::All => {cleanup::remove_all(&mut res); },
            }
    }

    else if let Some(regex) = &ARGS.print_id
    {
        let re = Regex::new(&regex).unwrap();
        for (id,resource) in &res.resources
        {
            if re.is_match(id)
            {
                println!("{:#?}\n",resource);
            }
        }
    }
    else if let Some(regex) = &ARGS.print_name 
    {
        let re = Regex::new(&regex).unwrap();
        for (_,resource) in &res.resources
        {
            if re.is_match(&resource.get_filename())
            {
                println!("{:#?}\n",resource);
            }
        }

    }
    else if let Some(ManifestOptions::Yaml) =  ARGS.manifest
    {
         println!("{}",res.as_yaml());
    }
    
    else if let Some(ManifestOptions::Json) =  ARGS.manifest
    {
         println!("{}",serde_json::to_string(&res).unwrap());
    }
    else if ARGS.generate
    {
        generate::generate(&mut res);
    }






    cleanup::cleanup(&mut res);
    res.write_resources();
}

