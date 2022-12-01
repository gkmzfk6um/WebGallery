use crate::datamodel::{Dependencies,Resources,Resource,ResourceProvider,DependencyType,DependencyFuncName};
use std::vec::Vec;
use std::collections::HashMap;
use std::sync::Mutex;

pub type DependencyFunc = Box<dyn Fn(&Resource) -> bool + Send + Sync>;
pub type DependencyData = Mutex<HashMap<DependencyFuncName,DependencyFunc>>;
use sha2::{Sha256, Digest};

lazy_static! {
    static ref DEPENDENCY_FUNCS : DependencyData = {
        let funcs : DependencyData = Default::default();
        {
            let mut data = funcs.lock().unwrap();
            for (name,f) in crate::generate::register_deps()
            {
                data.insert(name,f);
            }
        }
        funcs
    };
}


fn is_dep_outdated(resources: &Resources, dependencies : &HashMap<String,String> ) -> bool
{
    for (dep_name, dep_rev)  in dependencies.iter()
    {
        match   resources.resources.get(dep_name)
        {
            Some(dependency) =>
            {
                if dependency.content_hash != *dep_rev
                {
                    return true;
                }
            },
            None => {return true; }
        };
    }
    return false;
}

impl Dependencies
{
    pub fn new() -> Dependencies
    {
        Dependencies {
            dependencies: Default::default(),
            dep_type: DependencyType::Direct
        }
    }
    pub fn new_glob( name : &DependencyFuncName, resources: &Resources) -> Dependencies
    {
        let DependencyFuncName(str_name) = name; 
        let funcs = DEPENDENCY_FUNCS.lock().unwrap();
        if let Some(f) = funcs.get(name)
        {
            Dependencies {
                dependencies: {
                    let mut deps : HashMap<String,String> = Default::default();
                    resources.resources
                    .values()
                    .filter_map( |x| if f(&x)  {Some((x.id.clone(), x.content_hash.clone()))} else {None} ) 
                    .for_each(|(name,hash) | { deps.insert(name,hash); });
                    deps
                },
                dep_type: DependencyType::Glob(DependencyFuncName(str_name.clone()))
            }
        }
        else {
            panic!("No glob dependency with name {:#?} exists!",name)
        }
    }

    pub fn get_dependencies(&self) -> &HashMap<String,String>
    {
        &self.dependencies
    }


    pub fn hash_deps(&self) -> String {
        assert!(self.dependencies.len() > 0);
        let mut hasher = Sha256::new();
        for (name,hash) in self.dependencies.iter()
        {
            hasher.update(name.as_bytes());
            hasher.update(hash.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }


    pub fn is_valid(&self) -> bool
    {
        match &self.dep_type
        {
            DependencyType::Direct => true,
            DependencyType::Glob(name) => {
                let funcs = DEPENDENCY_FUNCS.lock().unwrap();
                funcs.contains_key(&name)
            }
        }
    }

    pub fn is_outdated(&self, resources: &Resources) -> bool
    {
        
        match &self.dep_type {
            DependencyType::Direct => is_dep_outdated(resources,&self.dependencies),
            DependencyType::Glob(f_name) => {
                let funcs = DEPENDENCY_FUNCS.lock().unwrap();
                match funcs.get(&f_name)
                {
                    None => panic!("No glob dependency with name {:#?} exists!",f_name),
                    Some(f) => {
                        for x in resources.resources.values()
                        {
                            let had_dep = self.dependencies.contains_key(&x.id);
                            let  has_dep = f(&x);
                            if had_dep != has_dep
                            {
                                return true;
                            }
                        }
                        is_dep_outdated(resources,&self.dependencies)
                    }
                }
            }
        }
    }

    pub fn depends_on(&self, resource: &Resource) -> bool
    {
        self.dependencies.get(&resource.id).is_some()
    }

    pub fn add_dependency(&mut self, resource: &Resource)
    {
        match self.dep_type{
            DependencyType::Direct =>  self.dependencies.insert(resource.id.clone(), resource.content_hash.clone()),
            DependencyType::Glob(_) => panic!("Can't add dependencies manually to dependency of glob type!")
        };
    }

}

pub fn reverse_dependencies(resource : & Resource, resources : & Resources ) -> Vec<String>
{
    resources.resources.values()
    .filter_map( |x| 
        if let ResourceProvider::Generated(y)= &x.resource_provider { 
            if y.depends_on(resource) { 
                Some(String::from(&x.id))
            }
            else 
            {
                None
            } 
        }
        else {
            None
        }
    )
    .collect()
}