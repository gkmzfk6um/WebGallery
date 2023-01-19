use tera::{Context,Tera};
use content_managment_datamodel::datamodel::{Resources,ResourceData,Resource,GeneratedDataDesc,ImageMetadata};
use crate::ARGS;
use std::collections::HashMap;
use std::fs::File;
use chrono::{Datelike};
use crate::datamodel::resource_file_manager::ResourceFileManager;
use std::cmp::Ordering;

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

lazy_static! {
    static ref GIT_SHA: String =  std::env::var("SOURCE_COMMIT").expect(format!("Environment variable SOURCE_COMMIT must be set to to generate HTML").as_str());
}



fn sort_by_years(resources: &Resources) -> Vec<(i32,Vec<&Resource>)>
{
    
    let mut images : Vec<(i32,&Resource)> = 
        resources.resources
        .values()
        .filter_map( | x :  &Resource| {
            match x.resource_data.to_value::<ImageMetadata>() 
            {
                Some(metadata) => Some( (metadata.date.year(),x) ),
                None => None
            }
    }).collect();
    images.sort_by(
        | (y1,r1),  (y2,r2) | -> Ordering
        {
            if y1 == y2
            {
                r1.as_data::<ImageMetadata>().date.cmp(&r2.as_data::<ImageMetadata>().date)
            }
            else 
            {
                y2.cmp(y1)
            }
        }
    );


    let mut result = Vec::new();
    let mut it = images.drain(..).peekable();
    while let Some( (y,r) ) = it.next()
    {
            let mut v = vec![r];
            loop 
            {
                if let Some( (y2, _) ) = it.peek()
                {
                    if  &y == y2 
                    {
                        let (_,r) = it.next().unwrap();
                        v.push(r);
                        continue;
                    }
                }
                break;
            }
            result.push((y,v));
    }
    result
    
    
}
fn sort_by_categories<'a>(resources: &'a Resources, categories :  &HashMap<String,Vec<String>> ) -> HashMap<String,Vec<&'a Resource>>
{
    
    let mut map = HashMap::new();
    for (category,ids) in categories
    {
        let mut res = Vec::new();
        res.reserve(ids.len());
        for id in ids
        {
            if let Some(r) = resources.resources.get(id)
            {
                res.push(r);
            }
            else 
            {
                println!("Could not find id {} in inventory!", id);
            }
        }
        if res.len() > 0 
        {
            map.insert(category.clone(),res);
        }
    }
    map
}

fn create_context(resources: &Resources)-> Context
{
    let mut context = Context::new();
    context.insert("resources",&resources.resources);
    
    let images :  Vec<&Resource> =
    resources.resources
        .values()
        .filter( |x : &&Resource | -> bool {
            match x.resource_data 
            {
                ResourceData::Image(_) => true,
                _ => false
            } 
        })
        .collect();
    context.insert("images",&images);
    context.insert("websiteName",&*crate::WEBSITE_NAME);
    context.insert("gitSha",GIT_SHA.as_str());
    context.insert("year", &chrono::offset::Utc::now().year() );
    context.insert("sortedbyyears",&sort_by_years(resources));
    if let Some(categories) = resources.find_data(|x : &GeneratedDataDesc| x.name == "categories" )
    {
        let f = File::open(categories.file_path()).expect(&format!("Could not open {:#?}",categories));
        match serde_json::from_reader(f)
        {
            Ok::<HashMap<String,Vec<String>>, _>(val) => {
                context.insert("sortedbycategories",&sort_by_categories(resources,&val));
            },
            Err(e) => {
                println!("Failed to read {:#?}\n{:#?}",categories,e);
            }
        }

    }
    else 
    {
        println!("Could not find categories!");
    }
    if let Some(prints) =  resources.find_data( |x : &GeneratedDataDesc| x.name == "prints" )
    {
        let f = File::open(prints.file_path()).expect(&format!("Could not open {:#?}",prints));
        match serde_json::from_reader(f)
        {
            Ok::<serde_json::Value, _>(val) => {
                context.insert("storeData", &val);
            },
            Err(e) => {
                println!("Failed to read {:#?}\n{:#?}",prints,e);
            }
        }
    }
    else {
        println!("Could not find print.json");
    }



    return context;
}




fn register_function(tera : &mut Tera)
{
    fn get_resource(args : &HashMap<String,tera::Value>) -> Resource
    {
        serde_json::from_value(args.get("resource").unwrap().clone()).unwrap()
    }
    fn get_image_name<'a>(image: &'a Resource) -> std::borrow::Cow<'a,str>
    {
        urlencoding::encode(&image.resource_data.to_value::<ImageMetadata>().unwrap().name)
    }

    
    tera.register_function("resource_to_print_url", 
        Box::new(move |args : &HashMap<String,tera::Value> | -> Result<tera::Value, tera::Error> {
            let resource = get_resource(args);
            let name = get_image_name(&resource);
            Ok(tera::Value::String(format!("/store/print/{}",name)))
        })
    );
    
    tera.register_function("resource_to_url", 
        Box::new(move |args : &HashMap<String,tera::Value> | -> Result<tera::Value, tera::Error> {
            let resource : Resource = serde_json::from_value(args.get("resource").unwrap().clone()).unwrap();
            let path = resource.get_path_relative_root().unwrap();
            let mut url_path = String::from("");
            for component  in path.components()
            {
                match component
                {
                    std::path::Component::Normal(s) => {
                        url_path.push('/'); 
                        url_path.push_str(&urlencoding::encode(&s.to_str().unwrap()))
                    },
                    _ => panic!("Found unexpected component of path {}",path.display())
                }
            }

            Ok(tera::Value::String(url_path))
        })
    );

    
}


pub fn generate(resources: &mut Resources)
{
    let glob_pattern = ARGS.root.join("static-templates/**");
    let glob_pattern_str = glob_pattern.to_str().unwrap();
    println!("tera glob: {}", glob_pattern_str);
    let mut tera = match Tera::new(glob_pattern_str) {
        Ok(t) => t,
        Err(e) => {
            panic!("Parsing error(s): {}", e);
        }
    };
    register_function(&mut tera);
    tera.autoescape_on(vec![]);

    let context = create_context(resources);


    for template in tera.get_template_names(){
        if !template.ends_with(".jinja2") && !resources.resources.contains_key(&template_id(template))
        {
            let f = File::create(template_path(template)).unwrap();
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
           data.name == "categories" ||
           data.name == "print.json"
        }
        _ => false
    }
}