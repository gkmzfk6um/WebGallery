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
    
#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
pub struct InfoVariant 
{
    pub width: u32,
    pub height: u32,
    pub price: u32
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
pub struct InfoItem 
{
    pub name: String,
    pub variants: HashMap<String,InfoVariant>
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
pub struct InfoResponse 
{
    pub failed: Vec<String>,
    pub success: HashMap<String,InfoItem>
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone,Default)]
pub struct CheckoutVariant 
{
    pub height: u32,
    pub width: u32,
    pub signature: u8,
    pub name: String
}
#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone,Default)]
pub struct CheckoutItem 
{
    pub id: String,
    pub quantity: u8,
    pub variant: CheckoutVariant
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone,Default)]
pub struct CheckoutCart 
{
    pub items: HashMap<String,CheckoutItem>,
    pub version: u32,
}
