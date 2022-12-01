use crate::datamodel::{DependencyFuncName,Resources,Resource,ResourceData,ResourceProvider,Dependencies,GeneratedDataDesc};
use std::vec::Vec;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap,HashSet};
use std::fs::File;
use xmp_toolkit::xmp_ns::{DC};
use std::io::Write;
use crate::ARGS;
use std::path::Path;

pub const CATEGORIES_RES_ID : &str = "generated_data_resource:computed_categories";
pub const CATEGORY_DEP_FUNC_NAME : &str =  "category_dep_func";

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
enum FilterMode{
    #[serde(rename = "or")]
    Or,
    #[serde(rename = "and")]
    And
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
struct CategoryFilter {
    namespace: Option<String>,
    path: String,
    value: String,
    #[serde(default)] 
    trace: bool
}

impl CategoryFilter {
    fn eval(&self,resources: &Resources) -> HashSet<String>
    {
        let mut matches :HashSet<String> = Default::default();
        let images = resources.resources.values().filter(|x| match x.resource_data
            {
                ResourceData::Image(_) => true,
                _ => false
            }
        );

        let namespace  = self.namespace.as_ref().map(|x| x.as_str() ) .unwrap_or(DC);

        for image in images 
        {

            let mut xmp_file = xmp_toolkit::XmpFile::new().unwrap();
            match xmp_file.open_file(&image.get_path(),xmp_toolkit::OpenFileOptions::default().for_read())
            {
                Ok(()) => { 
                    match xmp_file.xmp()
                    {
                        Some(meta) => {

                            if let Some(value) = meta.property(namespace,self.path.as_str())
                            {
                                if self.trace
                                {
                                    println!("{} {} {} == {}",namespace,self.path,value,self.value.as_str());
                                }
                                if value == self.value.as_str()
                                {
                                    matches.insert(String::from(image.id()));
                                }
                                else 
                                {
                                    let mut index : usize = 1;
                                    while let Some(value) = meta.property(namespace,format!("{}[{}]",self.path,index).as_str())
                                    {
                                        if self.trace
                                        {
                                            println!("{} {}[{}] {} == {}",namespace,self.path,index,value,self.value.as_str());
                                        }
                                        if value == self.value.as_str()
                                        {
                                            matches.insert(String::from(image.id()));
                                            break;
                                        }
                                        index+=1;
                                    }   
                                }
                            }
                            else {
                                if self.trace {
                                    println!("{} {} not present!",namespace,self.path );
                                }

                            }
                        },
                        None => {
                            if self.trace {
                                println!("No metadata in {}", image.get_path().display() );
                            }
                        }
                        
                    }
                },
                Err(e) => { println!("{:#?}",e); }
            }
        }

        matches
    }
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
struct Category {
    mode : FilterMode,
    filters: Vec<CategoryFilter>,
    exclusion: Option<Vec<CategoryFilter>>
}

impl Category {
    fn eval(&self, resources: &Resources) -> HashSet<String>
    {

        let sets = self.filters.iter().map(|x| x.eval(resources));

        let matches = match self.mode {
            FilterMode::Or => {sets.fold(HashSet::new(), |a, b| a.union(&b).map(|x| x.clone() ).collect()) }
            FilterMode::And => { 
                match sets.reduce(|a,b| a.intersection(&b).map(|x| x.clone()).collect()) 
                {
                    Some(matches) => matches,
                    None => Default::default()
                }
            
            }
        };

        if let Some(exclusion_filters) = &self.exclusion
        {
            if let Some(exclusions) = exclusion_filters.iter().map(|x| x.eval(resources)).reduce(|a,b| a.union(&b).map(|x| x.clone()).collect())
            {
                return matches.difference(&exclusions).map(|x| x.clone()).collect()
            }
        }
        return matches
    }
}


pub fn find_categories_config_res(resources: &mut Resources) -> Option<std::path::PathBuf>
{
    resources.resources.values().find_map( |x| 
        match &x.resource_data
        {
            ResourceData::Sitedata(data) => {
                if data.filename == "categories.json"
                {
                    Some(x.get_path().to_path_buf())
                }
                else {
                    None
                }
            },
            _ => None
        }
    )
}


pub fn generate(resources: &mut Resources)
{
    if !resources.resources.contains_key(CATEGORIES_RES_ID)  
    {
        if let Some(input_path) =  find_categories_config_res(resources)
        {
            let data = std::fs::read_to_string(input_path).expect("Unable to categories.json");
            let res: HashMap<String,Vec<Category>> = serde_json::from_str(&data).expect("Unable to parse");


            println!("Generating categories...");
            let root_relative_path = Path::new("resources/data/computed_categories.json");
            let path =ARGS.root.join(&root_relative_path) ;

            let json = {
                let mut categories : HashMap<String,HashSet<String>> = Default::default();
                for (category_name,filters) in res.iter()
                {
                    let mut matches = HashSet::new();
                    for filter in filters {
                        for id in filter.eval(resources){
                            matches.insert(id);
                        }
                    }
                    categories.insert(category_name.clone(), matches);
                } 
                serde_json::to_string(&categories).expect("Failed to serialize categories data")
            };

            

            let mut json_file = File::create(&path).unwrap();
           json_file.write_all(json.as_bytes()).unwrap(); 


            let deps = Dependencies::new_glob(&DependencyFuncName(String::from(CATEGORY_DEP_FUNC_NAME)), resources);
            let hash = deps.hash_deps();
            let provider = ResourceProvider::Generated(deps);

            let resource = Resource::new(root_relative_path, ResourceData::GeneratedData(GeneratedDataDesc {name: String::from("categories") } )  ,CATEGORIES_RES_ID, hash.as_str(), provider );
            resources.resources.insert(String::from(CATEGORIES_RES_ID), resource);
        }
        else 
        {
            println!("No categories .json found!")
        }
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