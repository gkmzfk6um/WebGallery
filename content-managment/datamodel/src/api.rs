use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
pub struct PrintApi
{
    pub variants: HashMap<String,crate::print::Variant>,
    pub description: String,
    pub brief: String,
    pub image: crate::datamodel::Resource,
    pub thumbnails: HashMap<crate::datamodel::ThumbnailSize ,crate::datamodel::Resource>
}