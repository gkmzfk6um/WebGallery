mod api;
mod diff;

use colored::Colorize;
use crate::datamodel::resource_file_manager::ResourcesFileManager;
use content_managment_datamodel::datamodel::Resources;
use crate::dropbox::api::fetch_resources;
use crate::dropbox::diff::{diff_manifest_and_inventory};
use crate::config::ResourceFilter;
use indicatif::ProgressBar;

const DROPBOX_FOLDER : &str = "/Photography/Published_dev";

pub async fn populate_using_dropbox(resources : Resources) -> Resources 
{
    let client = reqwest::Client::new();
    let mut manifest = api::get_manifest(&client).await.unwrap();
    let mut res = Resources {
        resources: resources.resources
    };

    let mut data_ops = diff_manifest_and_inventory(&res,&manifest, ResourceFilter::Data);
    let mut image_ops = diff_manifest_and_inventory(&res,&manifest,ResourceFilter::Images);

    println!("------------- Dropbox clone ---------------------");    
    println!("{0: <10} | {1: <10} | {2: <10} | {3: <10}", "Type", "Get", "Remove", "Unchanged");    
    println!("---------------------------------------------");    
    println!("{0: <10} | {1: <10} | {2: <10} | {3: <10}", "Data", format!("{}",data_ops.add.len()).green(), data_ops.remove.len(), data_ops.noop.len());    
    println!("{0: <10} | {1: <10} | {2: <10} | {3: <10}", "Images", format!("{}",image_ops.add.len()).green(), image_ops.remove.len(), image_ops.noop.len());    
    println!("---------------------------------------------");    

    data_ops.remove.append(&mut image_ops.remove);

    if data_ops.remove.len() > 0 
    {   //Clean files to be deleted

        println!("Removing stale dropbox files...");
        let bar = ProgressBar::new(image_ops.remove.len().try_into().unwrap());
        for deleted_id in data_ops.remove 
        {
            res.remove_resource(&deleted_id);
            bar.inc(1);
        }
        bar.finish();
    }

    {  //fetch config files
        let mut v = Vec::with_capacity(data_ops.add.len());
        for id in data_ops.add   
        {
            v.push(manifest.files.remove(&id).unwrap());
        }
        for resource in fetch_resources(&client,v,&ResourceFilter::Data).await
        {
            res.resources.insert(String::from(resource.id()),resource);
        }
    }
    
    {  //fetch images
        let mut v = Vec::with_capacity(image_ops.add.len());
        for id in image_ops.add   
        {
            v.push(manifest.files.remove(&id).unwrap());
        }
        for resource in fetch_resources(&client,v,&ResourceFilter::Images).await
        {
            res.resources.insert(String::from(resource.id()),resource);
        } 
    }

    return res;
}