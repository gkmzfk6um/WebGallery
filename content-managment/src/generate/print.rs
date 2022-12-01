use crate::datamodel::{DependencyFuncName,Resources,Resource,ResourceData,ResourceProvider,Dependencies,GeneratedDataDesc,ImageMetadata};
use std::vec::Vec;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap};
use std::fs::File;
use std::io::Write;
use crate::ARGS;
use std::path::Path;
use std::string::String;

pub const PRINT_RES_ID : &str = "generated_data_resource:prints";
pub const PRINT_DEP_FUNC_NAME : &str =  "print_dep_func";

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
struct Price{
    value: u32,
    cur: String
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
struct Variant{
    price: Price,
    width: u32,
    height: u32
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
struct PrintRaw
{
    name: String,
    variants: Vec<String>,
    description: Vec<String>,
    brief: String
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
struct PrintCompiled
{
    name: String,
    variants: Vec<String>,
    description: String,
    id: String,
    brief: String
}

#[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
struct PrintDefinition<PrintType>
{
    variants:HashMap<String,Variant>,
    prints:  HashMap<String,Vec<PrintType>>
}
    

pub fn find_print_config_res(resources: &mut Resources) -> Option<std::path::PathBuf>
{
    resources.resources.values().find_map( |x| 
        match &x.resource_data
        {
            ResourceData::Sitedata(data) => {
                if data.filename == "print.json"
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


fn compile_print_file(resources: &Resources, print_input: &PrintDefinition<PrintRaw> ) -> PrintDefinition<PrintCompiled>
{

    let mut output = PrintDefinition::<PrintCompiled> {
        variants:  print_input.variants.clone(),
        prints: std::default::Default::default()
    };


    for (category, prints) in &print_input.prints
    {

        let mut compiledCategory : Vec<PrintCompiled> = std::default::Default::default();
        'print_loop: for print in prints
        {
            for variant in &print.variants
            {
                if !print_input.variants.contains_key(variant)
                {
                    println!("Print: Variant {} no found!", variant);
                    continue 'print_loop;
                }
            }
            
            let mut description = String::from("");
            for description_file in &print.description
            {
                let path = ARGS.root.join(format!("resources/data/{}",description_file));
                if path.is_file()
                {
                    match std::fs::read_to_string(&path)
                    {
                        Ok(s) => description += &s,
                        Err(_) => {
                            println!("Print: Failed to open {}", description_file);
                            continue 'print_loop;
                        }
                    }
                }
                else 
                {
                    println!("Print: File does not exist {}", description_file);
                    continue 'print_loop;
                }
            }

            match resources.find_resource( |x : &Resource |  { x.get_filename() == print.name})
            {
                Some(res) => 
                {
                    let compiled = PrintCompiled {
                        name: print.name.clone(),
                        variants: print.variants.clone(),
                        description: description,
                        id: res.id().to_string(),
                        brief: print.brief.clone()
                    };
                    compiledCategory.push(compiled);

                },
                None => {
                    println!("Print image with name {} not found", print.name);
                    continue 'print_loop;
                }
            }
        }

        if compiledCategory.len() > 0
        {
            output.prints.insert(category.to_string(),compiledCategory);
        }
        else {
            println!("Print: Category {} empty!",category);
        }
    }

    return output;
}

pub fn generate(resources: &mut Resources)
{
    if !resources.resources.contains_key(PRINT_RES_ID)  
    {
        if let Some(input_path) =  find_print_config_res(resources)
        {
            let data = std::fs::read_to_string(input_path).expect("Unable to print.json");
            let mut print_data: PrintDefinition<PrintRaw>= serde_json::from_str(&data).expect("Unable to parse");


            let root_relative_path = Path::new("resources/data/computed_print.json");
            let path =ARGS.root.join(&root_relative_path) ;

            let json = {
                serde_json::to_string(&compile_print_file(&resources,&print_data)).expect("Failed to serialize categories data")
            };

            

            let mut json_file = File::create(&path).unwrap();
            json_file.write_all(json.as_bytes()).unwrap(); 


            let deps = Dependencies::new_glob(&DependencyFuncName(String::from(PRINT_DEP_FUNC_NAME)), resources);
            let hash = deps.hash_deps();
            let provider = ResourceProvider::Generated(deps);

            let resource = Resource::new(root_relative_path, ResourceData::GeneratedData(GeneratedDataDesc {name: String::from("prints") } )  ,PRINT_RES_ID, hash.as_str(), provider );
            resources.resources.insert(String::from(PRINT_RES_ID), resource);
        }
        else 
        {
            println!("No categories .json found!")
        }
    }
}



pub fn print_dep_func(resources : &Resource) -> bool {
    match &resources.resource_data {
        ResourceData::Image(_) => true,
        ResourceData::Sitedata(data) => {
            data.filename == "print.json"
        },
        _ => false
    }
}


