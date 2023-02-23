use content_managment_datamodel::datamodel::{Resources,Resource,ResourceData::Image,Dependencies,ThumbnailSize,ImageVariant,ResourceProvider,ResourceData};
use crate::datamodel::resource_file_manager::ResourceFileManager;
use image::io::Reader as ImageReader;
use rayon::prelude::*;
use crate::ARGS;
use std::path::Path;

use indicatif::ProgressBar;
use img_parts::{ImageICC};
use img_parts::jpeg::Jpeg;



pub fn get_image_size(size: &ThumbnailSize) -> u32
{
    match size {
        ThumbnailSize::Small  => 256,
        ThumbnailSize::Medium => 512,
        ThumbnailSize::Large  => 2048,
        ThumbnailSize::Huge   => 3000

    }
}

const NUMBER_OF_THUMBNAILS : usize =  4;



pub fn generate_thumbnail(image : &Resource, size: &ThumbnailSize, image_data: image::DynamicImage, icc_profile: Option<img_parts::Bytes>) -> Resource
{

    let mut deps = Dependencies::new_default();
    let id = format!("{}-thumbnail-{}",image.id(),size);
    let filename = format!("{}-thumbnail-{}.jpg", image.file_path().file_stem().unwrap().to_str().unwrap() ,size);
    let path =ARGS.root.join(Path::new("resources/thumbnails")).join(Path::new(&filename));
    deps.add_dependency(&image);


    let data = ResourceData::Thumbnail(ImageVariant {
        size: size.clone(),
        width: image_data.width(),
        height: image_data.height()
    });

    {

        let mut buffer = std::io::Cursor::new(Vec::new()); 
        image_data.write_to(&mut buffer, image::ImageOutputFormat::Jpeg(80)).unwrap();
        let mut jpeg = Jpeg::from_bytes(buffer.into_inner().into()).unwrap();

        jpeg.set_icc_profile(icc_profile);

        let output_file = std::fs::File::create(&path).unwrap();
        jpeg.encoder().write_to(output_file).unwrap();
    }

    
    let resource = Resource::new(path, data,&id, &image.content_hash, ResourceProvider::Generated(deps)  );
    resource.write_resource();
    resource
}


fn has_thumbnail(image : &Resource, size : &ThumbnailSize) -> bool
{
    match &image.resource_data
    {
        Image(i) => i.variants.contains_key(size),
        _ => panic!("Image {:#?} isn't a image!",image)
    }
}

pub fn generate(resources: &mut Resources)
{
    let valid_ids = resources.resources.keys().map(|x| x.clone()).collect();

    for  res in resources.resources.values_mut()
    {
        if let  Image(i) = &mut res.resource_data
        {
            i.prune(&valid_ids)
        }
    }

   let target_images : Vec<&mut Resource> =  resources.resources.values_mut().filter( 
        |v| if let Image(i) = &v.resource_data {
            i.variants.len() != NUMBER_OF_THUMBNAILS
        } else  {false} ).collect();

    if target_images.len() > 0 
    {
        let n_thumbnails = NUMBER_OF_THUMBNAILS*target_images.len();
        println!("Generating {} thumbnails...",n_thumbnails);
        let bar = ProgressBar::new(n_thumbnails.try_into().unwrap());
        bar.tick();

        let handles : std::vec::Vec<Vec<Resource>> = target_images
        .into_par_iter()
        .map( |image| 
        {
            let mut image_thumbnails : Vec<Resource> = Vec::with_capacity(NUMBER_OF_THUMBNAILS);
            let icc_profile = {
                let input = std::fs::read(&image.file_path()).unwrap();
                let jpeg = Jpeg::from_bytes(input.into()).unwrap();
                jpeg.icc_profile()
            };

            if let None = icc_profile
            {
                println!("Image {} missing ICC profile!", image.file_path().display() );
            }

            match ImageReader::open(&image.file_path())
            {
                Ok(reader) => {
                        match reader.decode()
                        {
                            Ok(read_image) => {
                                for size in [ThumbnailSize::Small,ThumbnailSize::Medium,ThumbnailSize::Large,ThumbnailSize::Huge].iter()
                                {
                                    if !has_thumbnail(&image,&size)  
                                    {
                                        let image_size = get_image_size(&size);
                                        let resized_image = read_image.resize(image_size,image_size, image::imageops::FilterType::Lanczos3);
                                        
                                        let thumbnail = generate_thumbnail(&image,&size,resized_image, icc_profile.clone());
                                        match &mut image.resource_data
                                        {
                                            Image(i) => i.variants.insert(size.clone(),String::from(thumbnail.id())),
                                            _ => panic!("Resource must be image!")
                                        };

                                        image_thumbnails.push(thumbnail);
                                    }
                                    bar.inc(1);
                                }
                            },
                            Err(e) => {
                                println!("Failed to decode src image {:#?}",e);
                                println!("{:#?}",image);
                            }

                        }
                },
                Err(e) => {
                    println!("Failed to read src image {:#?}",e);
                    println!("{:#?}",image);
                }
            }
            image.write_resource();
            image_thumbnails
        }).collect();


       for thumbnails in handles 
       {
           for thumbnail in thumbnails
           {
               resources.resources.insert(String::from(thumbnail.id()), thumbnail);
           }
       }
       bar.finish();
    }




}