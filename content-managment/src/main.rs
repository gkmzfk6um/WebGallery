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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Source 
{
    Dropbox
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CleanTargets 
{
    Thumbnails,
    Data,
    All
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ManifestOptions 
{
    Yaml,
}

/// Web gallery file managment and content generator
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {

   /// Clean specified categories of files
   #[clap(arg_enum,short, long, value_parser )]
   clean: Option<CleanTargets>,

   // Fetch remote resources using provided method
   #[clap(arg_enum,short, long, value_parser)]
   source: Option<Source>,
   
   // Generate derived resources
   #[clap(short, long, value_parser,default_value="true")]
   generate: bool,
   
   // print manifest in specified format to std::out
   #[clap(arg_enum,short, long, value_parser)]
   manifest: Option<ManifestOptions>,

}




#[tokio::main(worker_threads = 6)]
async fn main() {

    let args = Cli::parse();
    let mut stored_resources = Resources::read_resources();
    cleanup::cleanup(&mut stored_resources);
    let mut res = populate_using_dropbox(stored_resources).await;
    
    match args.clean {
        Some(op) => {
            match op {
                CleanTargets::Thumbnails => {cleanup::remove_thumbnails(&mut res); },
                CleanTargets::Data => {cleanup::remove_data(&mut res); },
                CleanTargets::All => {cleanup::remove_all(&mut res); },
            }
        },
        None => ()
    } ;


    if args.generate
    {
        generate::generate(&mut res);
    }
    //println!("{:#?}",res);

    cleanup::cleanup(&mut res);

    match args.manifest
    {
        Some(ManifestOptions::Yaml) => println!("{}",res.as_yaml()),
        _ => ()
    };


    //match serde_json::to_string(&res)
    //{
    //    Ok(s) => println!("{}",s),
    //    Err(_) => println!("Fail!")
    //};
    res.write_resources();
}

