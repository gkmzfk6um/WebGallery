
use std::vec::{Vec};
use std::collections::HashSet;

use crate::dropbox::api::{DropboxManifest,DropboxManifestEntry};
use crate::datamodel::{Resources,Resource,ResourceData};
use crate::config::ResourceFilter;


#[derive(Debug)]
pub struct  Operation <>
{
    pub add :    Vec< String>,
    pub remove : Vec< String>,
    pub noop :   Vec< String>
}


const IMAGE_SUFFIXES : [& str;2] = [".jpeg",".jpg"];

lazy_static! {
    static ref IMAGE_PATH : String = format!("{}/images/", crate::dropbox::DROPBOX_FOLDER) ;
    static ref DATA_PATH  : String = format!("{}/data/", crate::dropbox::DROPBOX_FOLDER) ;

}

pub fn diff_manifest_and_inventory(resources :&Resources, manifest : &DropboxManifest , f : ResourceFilter  ) -> Operation
{

    let filter : &dyn Fn(&&DropboxManifestEntry) -> bool   = match f
    {
        ResourceFilter::Data   => &|x | {
            x.path_display.starts_with(&(DATA_PATH).to_string())
        },
        ResourceFilter::Images => &|x | {
            if x.path_display.starts_with(&(IMAGE_PATH).to_string())
            {
                let low =   x.name.to_ascii_lowercase();
                for suffix in IMAGE_SUFFIXES
                {
                    if low.ends_with(suffix)
                    {
                        return true;
                    }
                }
            }
            return false;
        }
    };
    
    let resource_filter : &dyn Fn(&&Resource) -> bool   = match f
    {
        ResourceFilter::Data   => &|x | {
           match x.resource_data
           {
                ResourceData::Sitedata(_) => true,
                _ => false
           }
        },
        ResourceFilter::Images => &|x | {
           match x.resource_data
           {
                ResourceData::Image(_) => true,
                _ => false
           }

        }
    };

    let manifeset_ids : HashSet<&str> = HashSet::from_iter(manifest.files.values().filter(filter).map(|x| x.id.as_str() ));
    let inventory_ids : HashSet<&str> = HashSet::from_iter(resources.resources.values().filter(resource_filter).map( |x| x.id() ));

    let mut to_remove : Vec<String> = Vec::from_iter(inventory_ids.difference(&manifeset_ids).map(|x| x.to_string() ));
    let mut to_add    : Vec<String> = Vec::from_iter(manifeset_ids.difference(&inventory_ids).map(|x| x.to_string() ));
    let mut to_nothing = Vec::new();
    
    for overlap  in inventory_ids.intersection(&manifeset_ids)
    {
        let id = String::from(*overlap);
        let manifest_hash = manifest.files.get(&id).unwrap().content_hash.as_str();
        let inventory_hash = resources.resources.get(&id).unwrap().content_hash.as_str();
        if manifest_hash != inventory_hash
        {
            to_add.push(id.to_string());
            to_remove.push(id.to_string());
        }
        else {
            to_nothing.push(id.to_string());
        }
    }

    Operation {
        add: to_add,
        remove: to_remove,
        noop: to_nothing
    }

}
