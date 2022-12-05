mod thumbnails;
mod categories;
mod templates;
mod print;

use crate::datamodel::{Resources,ResourceProvider,DependencyFuncName};
use crate::datamodel::dependency::{DependencyFunc};
use std::vec::Vec;
use indicatif::ProgressBar;

pub fn generate(resources: &mut Resources)
{

    loop {
        let outdated_resources : Vec<String> = 
         resources.resources.values()
        .filter_map(|x| 
            match &x.resource_provider{
                ResourceProvider::Generated(d) =>  if d.is_outdated(resources) {Some(String::from(x.id())) } else {None} ,
                _ => None
            }
        ).collect();

        if outdated_resources.len() > 0
        {
            println!("Removing outdated generated content...");
            let bar = ProgressBar::new(outdated_resources.len().try_into().unwrap());
            bar.tick();
            for outdated_resource in outdated_resources
            {
                bar.inc(1);
                resources.remove_resource(&outdated_resource);
            }
            bar.finish();
        }
        else {
            break;
        }
    }


    thumbnails::generate(resources);
    categories::generate(resources);
    print::generate(resources);
    templates::generate(resources);
}

pub fn register_deps() -> [(DependencyFuncName,DependencyFunc);3]
{
    [
        (DependencyFuncName(String::from(categories::CATEGORY_DEP_FUNC_NAME)), std::boxed::Box::new(categories::category_dep_func)),
        (DependencyFuncName(String::from(templates::TEMPLATES_DEP_FUNC_NAME)), std::boxed::Box::new(templates::templates_dep_func)),
        (DependencyFuncName(String::from(print::PRINT_DEP_FUNC_NAME)), std::boxed::Box::new(print::print_dep_func))
    ]
}