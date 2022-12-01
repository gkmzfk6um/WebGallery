mod datamodel;
mod dropbox;
mod config;
mod generate;
mod cleanup;
#[macro_use]
extern crate lazy_static;

use crate::datamodel::Resources;
use crate::dropbox::populate_using_dropbox;
use clap::{Parser,ValueEnum};
use std::fs;
use std::path::Path;
use regex::Regex;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum,Debug)]
pub enum Source 
{
    Dropbox
}

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
}

/// Web gallery file managment and content generator
#[derive(Debug,Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {

   /// Clean specified categories of files
   #[clap(arg_enum,short, long, value_parser )]
   clean: Option<CleanTargets>,

   // Fetch remote resources using provided method
   #[clap(arg_enum,short, long, value_parser)]
   source: Option<Source>,
   
   #[clap(short, long, value_parser)]
   print_id: Option<String>,
   
   #[clap(long, value_parser)]
   print_name: Option<String>,
   
   // Generate derived resources
   #[clap(short, long, value_parser,default_value="true")]
   generate: bool,
   
   // Create resource directory tree if it doesn't exist
   #[clap(long, value_parser,default_value="false")]
   create_dir: bool,

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
}

fn create_resource_folder()
{
    if ARGS.create_dir
    {
        let resource_dir = ARGS.root.join(&Path::new("resources"));
        fs::create_dir(&resource_dir).expect(format!("Could not create resources dir under {}",ARGS.root.display()).as_str());
        fs::create_dir(&resource_dir.join(&Path::new("data")      )).expect("Could not create data folder");
        fs::create_dir(&resource_dir.join(&Path::new("images")    )).expect("Could not create images folder");
        fs::create_dir(&resource_dir.join(&Path::new("meta")      )).expect("Could not create meta folder");
        fs::create_dir(&resource_dir.join(&Path::new("thumbnails"))).expect("Could not create thumbnails folder");
    }
}



#[tokio::main(worker_threads = 6)]
async fn main() {

    create_resource_folder();
    let mut stored_resources = Resources::read_resources();
    cleanup::cleanup(&mut stored_resources);
    let mut res = populate_using_dropbox(stored_resources).await;
    
    match ARGS.clean {
        Some(op) => {
            match op {
                CleanTargets::Thumbnails => {cleanup::remove_thumbnails(&mut res); },
                CleanTargets::Data => {cleanup::remove_data(&mut res); },
                CleanTargets::All => {cleanup::remove_all(&mut res); },
            }
        },
        None => ()
    } ;


    if ARGS.generate
    {
        generate::generate(&mut res);
    }

    cleanup::cleanup(&mut res);

    match &ARGS.print_id 
    {
        Some(regex) => 
        {
            let re = Regex::new(&regex).unwrap();
            for (id,resource) in &res.resources
            {
                if re.is_match(id)
                {
                    println!("{:#?}\n",resource);
                }
            }
        },
        _ => ()
    };
    
    match &ARGS.print_name
    {
        Some(regex) => 
        {
            let re = Regex::new(&regex).unwrap();
            for (_,resource) in &res.resources
            {
                if re.is_match(&resource.get_filename())
                {
                    println!("{:#?}\n",resource);
                }
            }
        },
        _ => ()
    };

    match ARGS.manifest
    {
        Some(ManifestOptions::Yaml) => println!("{}",res.as_yaml()),
        _ => ()
    };



    res.write_resources();
}

