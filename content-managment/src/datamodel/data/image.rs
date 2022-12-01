use crate::datamodel::ImageMetadata;
use xmp_toolkit;
use xmp_toolkit::{XmpError};
use xmp_toolkit::xmp_ns::{DC,EXIF};
use chrono::{DateTime};

#[derive(Debug)]
pub struct ImageMetadataError 
{
    #[allow(dead_code)]
    error: Option<String>,
    #[allow(dead_code)]
    xmp_error: Option<XmpError>
}

impl ImageMetadataError {
    fn new(str: String)->ImageMetadataError
    {
        ImageMetadataError {
            error: Some(str),
            xmp_error: None
        }
    }
}

impl From<XmpError> for ImageMetadataError {
    fn from(error: XmpError) -> Self {
        ImageMetadataError {
            error: None,
            xmp_error: Some(error) 
        }
    }    
}

impl ImageMetadata {
    pub fn new<T: AsRef<std::path::Path>>(filename: &str, path : T ) -> Result<ImageMetadata,ImageMetadataError>
    {
        let mut xmp_file = xmp_toolkit::XmpFile::new()?;
        xmp_file.open_file(&path,xmp_toolkit::OpenFileOptions::default().for_read())?;
        let xmp_meta = xmp_file.xmp().ok_or( ImageMetadataError::new(format!("No XMP in file {}!",path.as_ref().display())))?;

        let  xmp_title = match xmp_meta.property(DC, "dc:title[1]")
        {
            Some(name) => name,
            None =>  { println!("No XMP title for {}",path.as_ref().display()); String::from(filename)} 
        };

        let parsed_xmp_date = 
        {
            let xmp_date = match xmp_meta.property(EXIF,"exif:DateTimeDigitized")
            {
                Some(date) => Some(date),
                None => match xmp_meta.property(EXIF,"exif:DateTimeOriginal")
                {
                    Some(date) => Some(date),
                    None => None
                }
            };

            if xmp_date == None
            {
               let metadata = std::fs::metadata(&path);
               if let Err(_) = metadata
               {
                    return Err(ImageMetadataError::new(String::from(format!("Failed to open file {} to read timestamp",path.as_ref().display()))));

               }
               if let Ok(time) = metadata.unwrap().created() {
                   time.into()
               } else {
                   return Err(ImageMetadataError::new(String::from("Failed to find image timestamp")));
               }
            }
            else 
            {
                let xmp_date = xmp_date.unwrap();
                match DateTime::parse_from_rfc3339(&xmp_date)
                {
                    Ok(date) => DateTime::from(date),
                    Err(_) => {
                        match chrono::NaiveDateTime::parse_from_str(&xmp_date,"%Y-%m-%dT%H:%M:%S")
                        {
                            Ok(date) => date.and_local_timezone(chrono::Utc).unwrap(),
                            _ => {
                                match chrono::NaiveDateTime::parse_from_str(&xmp_date,"%Y-%m-%dT%H:%M:%S%.f")
                                {
                                    Ok(date) => date.and_local_timezone(chrono::Utc).unwrap(),
                                    _ => return Err(ImageMetadataError::new(format!("Failed to parse {} XMP timestamp ({})",path.as_ref().display(),xmp_date)))
                                }
                            }

                        }
                    }
                }
            }
        };

        Ok(ImageMetadata {
            name:         xmp_title,
            date:         parsed_xmp_date,
            colour :      String::from(""),
            variants: Default::default()
        })
    }
}