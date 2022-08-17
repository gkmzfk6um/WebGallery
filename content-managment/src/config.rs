
#[derive(Debug, Copy, Clone,PartialEq,Eq)]
pub enum ResourceFilter 
{
    Data,
    Images
}

pub fn resource_path<T: std::fmt::Display>(name: T, filter : &ResourceFilter ) -> String
{
    match filter 
    {
        ResourceFilter::Data => format!("resources/data/{}",name),
        ResourceFilter::Images => format!("resources/images/{}",name)
    }
}