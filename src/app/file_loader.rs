use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use dicom::core::Tag;
use dicom::object::open_file;
use dicom::pixeldata::image::GenericImageView;
use dicom::pixeldata::PixelDecoder;
use iced::widget::image::Handle;

pub fn open_folder(path_buf: PathBuf) -> (Vec<String>, HashMap<String, PathBuf>) {

    let mut path_map:HashMap<String, PathBuf> = HashMap::new();
    let mut series_names:Vec<String> = Vec::new();

    if path_buf.is_dir() {
        for entry in fs::read_dir(path_buf).unwrap() {
            match entry{
                Ok(path) => {
                    let path = path.path();
                    if path.is_dir() {
                        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
                        path_map.insert(file_name.clone(), path);
                        series_names.push(file_name);
                    } else {
                        println!("{} is not a valid folder!", path.display());
                    }
                }
                Err(_) => {}
            }
        }
    }
    (series_names, path_map)
}
pub fn load_image(path_buf: PathBuf) -> Handle {

    let obj = open_file(path_buf).unwrap();

    let decoded_pixel_data = obj.decode_pixel_data().unwrap();
    let dyn_img = decoded_pixel_data.to_dynamic_image(0).unwrap();

    let (width, height) = dyn_img.dimensions();

    let rgba_img = dyn_img.to_rgba8();
    let rgba_vec = rgba_img.to_vec();

    Handle::from_rgba(width, height, rgba_vec)
}

pub fn load_images(path_buf: PathBuf) -> Vec<ImageSlice> {

    let mut image_vec = Vec::<ImageSlice>::new();
    let mut paths:Vec<PathBuf> = Vec::new();
    if path_buf.is_dir() {
        for entry in fs::read_dir(path_buf).unwrap() {
            match entry{
                Ok(path) => {
                    paths.push(path.path());
                }
                Err(_) => {}
            }
        }
    } else if path_buf.is_file() {
        paths = vec![path_buf];
    }

    for path in paths {

        let file_name = path.to_str().unwrap().to_owned();
        let obj = open_file(path).unwrap();

        let location  = match obj.element( Tag(0x0020,0x1041) ) {
            Ok(location) => Some(location.to_float32().unwrap()),
            Err(_e) => None,
        };

        let (rgba_vec, width, height) = match obj.decode_pixel_data() {
            Ok(decoded_pixel_data) => {
                let dyn_img = decoded_pixel_data.to_dynamic_image(0).unwrap();

                let (width, height) = dyn_img.dimensions();

                let rgba_img = dyn_img.to_rgba8();
                let rgba_vec = rgba_img.to_vec();

                (rgba_vec, width, height)
            },
            Err(_e) => {(Vec::new(),0,0)}
        };

        image_vec.push(ImageSlice {
            file_name,
            width,
            height,
            rgba_vec,
            location,
        });
    };

    image_vec.sort_by(|a, b| {
        if let Some(a_location)  = a.location {
            if let Some(b_location) = b.location {
                return b_location.total_cmp(&a_location);
            };
            return Ordering::Equal
        };
        return Ordering::Equal
    });
    image_vec
}

#[derive(Debug, Clone)]
pub struct ImageSlice {
    pub file_name: String,
    pub width: u32,
    pub height: u32,
    pub rgba_vec: Vec<u8>,
    location: Option<f32>,
}

impl ImageSlice {
    pub fn get_handle(self) -> Handle {
        Handle::from_rgba(self.width, self.height, self.rgba_vec)
    }
}