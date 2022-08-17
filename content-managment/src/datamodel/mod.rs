pub mod data;
mod resource_file_manager;
mod dependency;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::hash::{Hash,Hasher};
    

    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
    pub struct Dependencies {
        dependencies: std::collections::HashMap<String,String>
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
        pub variants: std::collections::HashMap<ThumbnailSize,String>
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
    pub enum ResourceData {
        Image(ImageMetadata),
        Thumbnail(ImageVariant),
        Sitedata(SiteDataConfig)
    }

    

    #[derive(Serialize, Deserialize,Eq,PartialEq,Debug,Clone)]
    pub struct Resource {
        pub date_created: DateTime<Utc>,
        pub id: String,
        pub content_hash: String,
        pub resource_provider: ResourceProvider,
        pub resource_data:  ResourceData,
        pub path: String
    }

    impl Resource
    {
        pub fn new(path : String, data : ResourceData , id: &str, content_hash : &str, provider : ResourceProvider ) -> Resource
        {
            Resource {
                date_created: Utc::now(),
                id: String::from(id),
                content_hash: String::from(content_hash),
                resource_provider: provider,
                resource_data : data,
                path 
            }
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
        pub resources : std::collections::HashMap<String,Resource>
    }
