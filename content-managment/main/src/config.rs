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
        ResourceFilter::Data =>   ARGS.root.join("resources/data/").join(name) ,
        ResourceFilter::Images => ARGS.root.join("resources/images/").join(name)
    }
}