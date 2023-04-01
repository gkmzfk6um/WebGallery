use crate::ARGS;
use std::path::{Path,PathBuf};

#[derive(Debug, Copy, Clone,PartialEq,Eq)]
pub enum ResourceFilter 
{
    Data,
    Images
}

pub fn resource_path<T: AsRef<Path>>(name: T, filter : &ResourceFilter ) -> PathBuf
{
    match filter 
    {
        ResourceFilter::Data => {
            match name.as_ref().extension()
            {
                Some(os_str) => 
                {
                        match os_str.to_str()
                        {
                            Some("jpg") => ARGS.root.join("resources/resource-images/").join(name),
                            Some("jpeg") => ARGS.root.join("resources/resource-images/").join(name),
                            Some(_) =>  ARGS.root.join("resources/data/").join(name),
                            None => panic!("Could convert os_str to string!")
                        }

                },
                _ => ARGS.root.join("resources/data/").join(name)
            }
        } 
        ResourceFilter::Images => ARGS.root.join("resources/images/").join(name)
    }
}