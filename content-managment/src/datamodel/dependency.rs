use crate::datamodel::{Dependencies,Resources,Resource,ResourceProvider};
use std::vec::Vec;

impl Dependencies
{
    pub fn new() -> Dependencies
    {
        Dependencies {
            dependencies: Default::default()
        }
    }
    pub fn is_outdated(&self, resources: &Resources) -> bool
    {
        for (dep_name, dep_rev)  in self.dependencies.iter()
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

    pub fn depends_on(&self, resource: &Resource) -> bool
    {
        self.dependencies.get(&resource.id).is_some()
    }

    pub fn add_dependency(&mut self, resource: &Resource)
    {
        self.dependencies.insert(resource.id.clone(), resource.content_hash.clone());
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