use std::fs;
use std::path::PathBuf;
use dicom::object::open_file;
use dicom::pixeldata::image::GenericImageView;
use dicom::pixeldata::PixelDecoder;
use iced::widget::image::Handle;
pub fn load_image(path_buf: PathBuf) -> Handle {

    let obj = open_file(path_buf).unwrap();

    let decoded_pixel_data = obj.decode_pixel_data().unwrap();
    let dyn_img = decoded_pixel_data.to_dynamic_image(0).unwrap();

    let (width, height) = dyn_img.dimensions() as (u32, u32);

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
        dbg!(&path);
        let file_name = path.to_str().unwrap().to_owned();
        let obj = open_file(path).unwrap();

        let decoded_pixel_data = obj.decode_pixel_data().unwrap();
        let dyn_img = decoded_pixel_data.to_dynamic_image(0).unwrap();

        let (width, height) = dyn_img.dimensions() as (u32, u32);

        let rgba_img = dyn_img.to_rgba8();
        let rgba_vec = rgba_img.to_vec();


        image_vec.push(ImageSlice {
            file_name,
            width,
            height,
            rgba_vec
        });
    };
    image_vec
}

#[derive(Debug, Clone)]
pub struct ImageSlice {
    pub file_name: String,
    width: u32,
    height: u32,
    rgba_vec: Vec<u8>,
}

impl ImageSlice {
    pub fn get_handle(self) -> Handle {
        Handle::from_rgba(self.width, self.height, self.rgba_vec)
    }
}