
use content_managment_datamodel::print::{PrintCompiled,PrintDefinition,Variant};
use content_managment_datamodel::datamodel::{Resource,Resources,ThumbnailSize,ImageMetadata};
use std::collections::HashMap;

pub fn get_variants(print : &PrintCompiled, print_definition : &PrintDefinition<PrintCompiled>) -> HashMap<String,Variant>
{
    print.variants
        .iter()
        .map( |x| (x.clone(), print_definition.variants.get(x).unwrap().clone()) )
        .collect()
}

pub fn get_thumbnails(image: &Resource ,resources : &Resources) -> HashMap<ThumbnailSize,Resource>
{
    image
        .as_data::<ImageMetadata>()
        .variants
        .iter()
        .map( |(k,v)| (k.clone(), resources.resources.get(v).unwrap().clone() )  )
        .collect()
}

pub fn find_print(print_id: &str, prints: &PrintDefinition<PrintCompiled>) -> Option<PrintCompiled>
{
    for (_category,category_prints) in &prints.prints
    {
        for print in category_prints 
        {
            if print.id == print_id
            {
                return Some(print.clone())
            }
        }
    }
    return None

}