use tera::{Context,Tera};
use crate::datamodel::{Resources,ResourceData,Resource};
use crate::ARGS;
use std::fs::File;

fn template_id(template_name: &str) -> String
{
    format!("tera-template:{}", template_name)
}

pub fn template_path(template_name: &str) -> std::path::PathBuf
{
    match template_name
    {
        _ => {
            ARGS.root.join(template_name)
        }
    }
}


pub fn generate(resources: &mut Resources)
{
    let glob_pattern = ARGS.root.canonicalize().unwrap().join("templates/**");
    let glob_pattern_str = glob_pattern.to_str().unwrap();
    println!("tera glob: {}", glob_pattern_str);
    let tera = match Tera::new(glob_pattern_str) {
        Ok(t) => t,
        Err(e) => {
            panic!("Parsing error(s): {}", e);
        }
    };
    let mut context = Context::new();
    context.insert("resources",resources);
    context.insert("gitSha","DUMMY_SHA");


    for template in tera.get_template_names(){
        if !template.ends_with(".jinja2") && !resources.resources.contains_key(&template_id(template))
        {
            let mut f = File::create(template_path(template)).unwrap();
            match tera.render_to(template,&context,f) 
            {
                Ok(_) => { println!("Rendered {}",template); },
                Err(e) => {panic!("{:#?}",e)}

            }
        }
    }


}


pub const TEMPLATES_DEP_FUNC_NAME: &str = "TERA_TEMPLATES_DEP";

pub fn templates_dep_func(resources : &Resource) -> bool {
    match &resources.resource_data {
        ResourceData::Image(_) => true,
        ResourceData::Thumbnail(_) => true,
        ResourceData::GeneratedData(data) => {
           data.name == "categories"
        }
        _ => false
    }
}