use crate::datamodel::SiteDataConfig;
use std::fs::File; 
#[derive(Debug)]
pub struct SiteDataConfigError {
    #[allow(dead_code)]
    error: String
}



impl SiteDataConfig
{
    pub fn new<T: AsRef<std::path::Path>> (_filename : &str,  path: T) ->Result<SiteDataConfig,SiteDataConfigError>
    {
        match File::open(&path)
        {
            Ok(_) => Ok(SiteDataConfig {
                filename: String::from(_filename)
            }),
            Err(_) => Err(SiteDataConfigError {error: format!("Failed to open resource file {}!",path.as_ref().display())})
        }
    }
}