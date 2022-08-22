use crate::datamodel::{Resource,Resources,ResourceData,ResourceProvider};
use std::path::{Path,PathBuf};
use std::collections::HashSet;
use indicatif::ProgressBar;
pub fn cleanup(resources : &mut Resources )
{
    let mut path_buffer : HashSet<PathBuf> = Default::default();


    clean_broken_resources(resources);
    clean_broken_dependencies(resources);
    
    resources.resources
    .values()
    .map( |x| {
        Path::new(&x.path).canonicalize().unwrap() 
    })
    .for_each(|x| {path_buffer.insert(x); });
    cleanup_folder("resources/data",&path_buffer );
    cleanup_folder("resources/images",&path_buffer );
    cleanup_folder("resources/thumbnails",&path_buffer );

}

pub fn clean_broken_resources(resources : &mut Resources )
{
    resources.resources.retain( |_, v| {
        if Path::new(&v.path).exists() {true} else {println!("Purging resource without valid path {:#?}",v); false }
    });
}

pub fn clean_broken_dependencies(resources: &mut Resources )
{
    resources.resources.retain( |_, v| {
        match &v.resource_provider
        {
            ResourceProvider::Generated(deps) => {if deps.is_valid() {true} else {println!("Purging resource with invalid dependency"); false}},
            _ => true
        }
    });
}


fn cleanup_folder(folder_path: &str, valid_paths: &HashSet<PathBuf> )
{
        let mut r = std::fs::read_dir(folder_path).unwrap();
        while let Some(Ok(dir)) = r.next()
        {
            let path = dir.path().canonicalize().unwrap();
            if !valid_paths.contains(&path)
            {
                println!("Purging stale file {}", path.to_string_lossy());
                std::fs::remove_file(path).unwrap();
            }
        }

}

fn remove_resources<F: Fn(&Resource) -> Option<String>> (resources : &mut Resources, f : F )
{
    let res_to_remove : std::vec::Vec<String> = resources.resources.values().filter_map(f).collect();
    let bar = ProgressBar::new(res_to_remove.len().try_into().unwrap());
    bar.tick();
    for res_name in res_to_remove
    {
        resources.remove_resource(&res_name);
        bar.inc(1);
    }
    bar.finish();
}

pub fn remove_thumbnails(resources : &mut Resources)
{
    println!("Cleaning thumbnails...");
    remove_resources(resources, 
        |x|
        match x.resource_data {
            ResourceData::Thumbnail(_) => {
                Some(String::from(&x.id))
            },
            _=> None
        }
    );
}

pub fn remove_data(resources : &mut Resources)
{
    println!("Cleaning data...");
    remove_resources(resources, 
        |x|
        match x.resource_data {
            ResourceData::Sitedata(_) => {
                Some(String::from(&x.id))
            },
            _=> None
        }
    );
}
pub fn remove_all(resources : &mut Resources)
{
    println!("Cleaning everything...");
    remove_resources(resources, 
        |x| Some(String::from(&x.id))
    );
}