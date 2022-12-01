use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::hash::{Hash,Hasher};
use std::collections::HashMap;
use std::path::{PathBuf,Path};

pub mod data;
pub mod dependency;
pub mod resource_file_manager;


    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone,Hash)]
    pub struct DependencyFuncName(pub String);



    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
    pub enum DependencyType 
    {
        Direct,
        Glob(DependencyFuncName)
    }

    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
    pub struct Dependencies {
        dependencies: HashMap<String,String>,
        dep_type: DependencyType
    }

    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
    pub enum ResourceProvider {
        Dropbox,
        Generated(Dependencies)
    }

    #[derive(Serialize, Deserialize,Eq,PartialEq,Hash,Clone,Debug)]
    pub enum ThumbnailSize 
    {
        Small,
        Medium,
        Large,
        Huge,
    }


    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
    pub struct ImageVariant {
        pub size: ThumbnailSize,
        pub width : u32,
        pub height: u32
    }



    impl std::fmt::Display for ThumbnailSize
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self
            {
                ThumbnailSize::Small => write!(f,"small"),
                ThumbnailSize::Medium =>write!(f, "medium"),
                ThumbnailSize::Large  =>write!(f, "large"),
                ThumbnailSize::Huge  =>write!(f, "huge")
            }
        }
    }

    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
    pub struct ImageMetadata {
        pub name:         String,
        pub date:         DateTime<Utc>,
        pub colour : String,
        pub variants: HashMap<ThumbnailSize,String>
    }

    
    impl ImageMetadata
    {
        pub fn prune(&mut self, valid_ids : &std::collections::HashSet<String>)
        {
            self.variants = self.variants.drain().filter(|(_,v)| valid_ids.contains(v) ).collect();
        }
    }


    impl Hash for ImageMetadata
    {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.name.hash(state);
        }
    }

    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
    pub struct SiteDataConfig {
        pub filename: String
    }
    
    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
    pub struct GeneratedDataDesc {
        pub name: String
    }


    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
    pub enum ResourceData {
        Image(ImageMetadata),
        Thumbnail(ImageVariant),
        Sitedata(SiteDataConfig),
        GeneratedData(GeneratedDataDesc),
        TeraTemplate(String)
    }
   
    pub trait ResourceDataType : Sized {
        fn convert<'a>(data: &'a ResourceData) -> Option<&'a Self>;
    }


    impl ResourceDataType for ImageMetadata {
        fn convert<'a> (data: &'a ResourceData) -> Option<&'a ImageMetadata>
        {
            match data {
                ResourceData::Image(s) => Some(s),
                _ => None
            }
        }
    }
    
    impl ResourceDataType for ImageVariant {
        fn convert<'a> (data: &'a ResourceData) -> Option<&'a ImageVariant>
        {
            match data {
                ResourceData::Thumbnail(s) => Some(s),
                _ => None
            }
        }
    }
    
    impl ResourceDataType for SiteDataConfig {
        fn convert<'a> (data: &'a ResourceData) -> Option<&'a SiteDataConfig>
        {
            match data {
                ResourceData::Sitedata(s) => Some(s),
                _ => None
            }
        }
    }
    
    impl ResourceDataType for GeneratedDataDesc {
        fn convert<'a> (data: &'a ResourceData) -> Option<&'a GeneratedDataDesc>
        {
            match data {
                ResourceData::GeneratedData(s) => Some(s),
                _ => None
            }
        }
    }

    impl ResourceData {
        pub fn to_value<'a, T>(&'a self) -> Option<&'a T>  where T : ResourceDataType
        {
            ResourceDataType::convert(self)
        } 
    }

    

    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
    pub struct Resource {
        date_created: DateTime<Utc>,
        id: String,
        pub content_hash: String,
        pub resource_provider: ResourceProvider,
        pub resource_data:  ResourceData,
        path: PathBuf
    }

    impl Resource
    {
        pub fn new<T: AsRef<Path> >(path : T, data : ResourceData , id: &str, content_hash : &str, provider : ResourceProvider ) -> Resource
        {
            let path = path.as_ref();
            Resource {
                date_created: Utc::now(),
                id: String::from(id),
                content_hash: String::from(content_hash),
                resource_provider: provider,
                resource_data : data,
                path: path.to_path_buf()
            }
        }

        pub fn id(&self) -> &str
        {
            self.id.as_str()
        }

        pub fn as_data<T: ResourceDataType>(&self) -> &T
        {
            self.resource_data.to_value().unwrap()
        } 
        pub fn try_data<T: ResourceDataType>(&self) -> Option<&T>
        {
            self.resource_data.to_value()
        } 

    }
    
    impl  Hash for Resource
    {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.id.hash(state);
        }
    }



    #[derive(Serialize, Deserialize,Eq,PartialEq,Default,Debug,Clone)]
    pub struct Resources
    {
        pub resources : HashMap<String,Resource>
    }

    impl Resources {

        pub fn find_resource<F>(&self, f : F )-> Option<&Resource> where F: Fn(&Resource) -> bool
        {
            self.resources.iter().find(|(_,r)| f(r) ).map(|(_,r)| r )
        }

        pub fn find_data<T: ResourceDataType,F>(&self, f :F ) -> Option<&Resource>  where  F: Fn(&T) -> bool 
        {
            for (_,resource) in &self.resources
            {
                match resource.resource_data.to_value::<T>()
                {
                    Some(t) => {
                        if f(t)
                        {
                            return Some(&resource)
                        }
                    },
                    None => {
                        continue;
                    }
                };
            } 
            return None;
        }
    }
