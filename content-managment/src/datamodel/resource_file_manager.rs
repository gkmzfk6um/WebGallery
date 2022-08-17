use std::fs::File;
use std::io::{Read,Write};
use crate::datamodel::{Resource,Resources};
use rmp_serde;
use lz4_flex;
use serde::{Serialize};
use std::fmt::Debug;
use crate::datamodel::dependency::reverse_dependencies;

impl Resources
{
    pub fn read_resources() -> Resources
    {
        let mut resources : Resources = Default::default();
        let mut r = std::fs::read_dir("resources/meta").unwrap();
        while let Some(Ok(dir)) = r.next()
        {
            let path = dir.path();
            if let Some(resource) = Resource::read_resource(&path)
            {
                resources.resources.insert(String::from(&resource.id),resource);
            }
        }
        resources
    }
    

    pub fn write_resources(&self)
    {
        for resource in self.resources.values()
        {
            resource.write_resource();
        }
    } 

    pub fn remove_resource(&mut self, id: &str)
    {
        match self.resources.remove(id)
        {
            Some(resource) => {
                for reverse_dep in reverse_dependencies(&resource,&self)
                {
                    self.remove_resource(&reverse_dep)
                }
                resource.delete_resource()
            },
            _ => {  println!("Failed to remove resource {}", id); }
        }
    }

    pub fn as_yaml(&self) -> String
    {
        serde_yaml::to_string(self).unwrap()
    }
}

fn delete_file<T: std::convert::AsRef<std::ffi::OsStr>>(path: T)
{
        let resource_path = std::path::Path::new(&path);
        if resource_path.exists()
        {
            std::fs::remove_file(resource_path).unwrap();
        }
}

impl Resource
{
    pub fn get_metadata_path(&self) -> String
    {
        format!("resources/meta/{}.binres",std::path::Path::new(&self.path).file_name().unwrap().to_str().unwrap())
    }

    pub fn delete_resource(&self)
    {
        delete_file(&self.path);
        delete_file(self.get_metadata_path());
    }

    pub fn write_resource(&self) {
        let path = self.get_metadata_path();
        let fut = File::create(&path);
        match fut
        {
            Ok(mut file) => 
            {
                let mut buf = Vec::new();
                let mut compressor = lz4_flex::frame::FrameEncoder::new(&mut buf);
                let s = self.serialize(&mut rmp_serde::Serializer::new(&mut compressor));
                match s {
                    Ok(_) => {
                        match compressor.finish()
                        {
                            Ok(_) => {
                                match (&mut file).write(&buf) 
                                {
                                    Ok(_) => (),
                                    Err(_) => panic!("Failed to write compressed data to file")
                                }
                            },
                            Err(_) => panic!("Failed to finnish compression")

                        }
                    },
                    Err(_)  => panic!("Failed to serialize data")
                };

            },
            Err(_) => panic!("Failed to open resource file {}",path )
        };
    }


    pub fn read_resource<T: std::convert::AsRef<std::path::Path> + std::convert::AsRef<std::ffi::OsStr>  + Debug>(path: &T) -> Option<Resource>
    {
        match File::open(path)
        {
            Ok(mut file) => {
                let mut buf = Vec::new();
                match file.read_to_end(&mut buf)
                {
                    Ok(_bytes) => {
                        let decompressor = lz4_flex::frame::FrameDecoder::new(std::io::Cursor::new(&buf));

                        match rmp_serde::decode::from_read(decompressor)
                        {
                            Ok(res) =>  return Some(res),
                            Err(e) => { println!("Failed to deserialize {:#?}\n {:#?}",path,e);  }
                        }
                    },
                    Err(_) => {
                        println!("Failed to read file {:#?}",path);
                    }
                }
            },
            Err(_) => {
                println!("No resource file {:#?}",path);
            }
        };
        delete_file(path);
        None
    }

}