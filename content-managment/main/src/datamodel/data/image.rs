use content_managment_datamodel::datamodel::ImageMetadata;
use xmp_toolkit;
use xmp_toolkit::{XmpError};
use xmp_toolkit::xmp_ns::{DC,EXIF};
use chrono::{DateTime};
use image::{ImageError,GenericImageView};
use image::io::Reader as ImageReader;

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

impl From<std::io::Error> for ImageMetadataError {
    fn from(error: std::io::Error) -> Self {
        ImageMetadataError {
            error: Some(format!("{:#?}",error) ),
            xmp_error: None,
        }
    }    
}
impl From<ImageError> for ImageMetadataError {
    fn from(error: ImageError) -> Self {
        ImageMetadataError {
            error: Some(format!("{:#?}",error) ),
            xmp_error: None,
        }
    }    
}

pub trait ImageMetadataCreator: Sized
{
    fn new<T: AsRef<std::path::Path>>(filename: &str, path : T ) -> Result<Self,ImageMetadataError>;
}

impl ImageMetadataCreator for ImageMetadata {
    fn new<T: AsRef<std::path::Path>>(filename: &str, path : T ) -> Result<ImageMetadata,ImageMetadataError>
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

        let hex_color = {
            let image=  ImageReader::open(path)?.decode()?;
            let rgb_img = image.into_rgb8();
            let (width,height) = rgb_img.dimensions();

            let (mut r,mut g,mut b) = (0u128,0u128,0u128);
            for (_,row) in rgb_img.enumerate_rows()
            {

            let (mut rr,mut rg,mut rb) = (0u128,0u128,0u128);
                for ( _ , _ , pixel) in row 
                {
                    rr+= u128::from(pixel[0]);
                    rg+= u128::from(pixel[1]);
                    rb+= u128::from(pixel[2]);
                }
                r+=rr/u128::from(width);
                g+=rg/u128::from(width);
                b+=rb/u128::from(width);
            }
            r/=u128::from(height);
            g/=u128::from(height);
            b/=u128::from(height);
                
           String::from(&format!("{:#08x}", r << 16 | g << 8 | b << 0)[2..])
        };
        if  hex_color.len() != 6
        {
             return Err( ImageMetadataError::new(format!("Hex failed {}",hex_color)) )
        }

        Ok(ImageMetadata {
            name:         xmp_title,
            date:         parsed_xmp_date,
            colour :      String::from(hex_color),
            variants: Default::default()
        })
    }
}