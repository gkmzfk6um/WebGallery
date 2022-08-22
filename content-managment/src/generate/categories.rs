use crate::datamodel::{DependencyFuncName,Resources,Resource,ResourceData,ResourceProvider,Dependencies,GeneratedDataDesc};


pub const CATEGORIES_RES_ID : &str = "generated_data_resource:computed_categories";
pub const CATEGORY_DEP_FUNC_NAME : &str =  "category_dep_func";

pub fn generate(resources: &mut Resources)
{
    if !resources.resources.contains_key(CATEGORIES_RES_ID)
    {
        println!("Generating categories...");
        let path = String::from("resources/data/computed_categories.bin");
        let hash = "compute_a_better_hash";

        let provider = ResourceProvider::Generated(Dependencies::new_glob(&DependencyFuncName(String::from(CATEGORY_DEP_FUNC_NAME)), resources));

        let resource = Resource::new(path, ResourceData::GeneratedData(GeneratedDataDesc {name: String::from("categories") } )  ,CATEGORIES_RES_ID, hash, provider );
        resources.resources.insert(String::from(CATEGORIES_RES_ID),resource);
    }

}



pub fn category_dep_func(resources : &Resource) -> bool {
    match &resources.resource_data {
        ResourceData::Image(_) => true,
        ResourceData::Sitedata(data) => {
            data.filename == "categories.json"
        },
        _ => false
    }
}