use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
pub struct Price{
    pub value: u32,
    pub cur: String
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
pub struct Variant{
    pub price: Price,
    pub width: u32,
    pub height: u32
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
pub struct PrintRaw
{
    pub name: String,
    pub variants: Vec<String>,
    pub description: Vec<String>,
    pub brief: String
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
pub struct PrintCompiled
{
    pub name: String,
    pub variants: Vec<String>,
    pub description: String,
    pub id: String,
    pub brief: String
}


#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone,Default)]
pub struct PrintDefinition<PrintType>
{
    pub variants:HashMap<String,Variant>,
    pub prints:  HashMap<String,Vec<PrintType>>
}

