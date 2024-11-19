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
    for path in path_buf.into_iter() {
        let obj = open_file(path).unwrap();

        let decoded_pixel_data = obj.decode_pixel_data().unwrap();
        let dyn_img = decoded_pixel_data.to_dynamic_image(0).unwrap();

        let (width, height) = dyn_img.dimensions() as (u32, u32);

        let rgba_img = dyn_img.to_rgba8();
        let rgba_vec = rgba_img.to_vec();

        image_vec.push(ImageSlice {
            width,
            height,
            rgba_vec
        });
    };
    image_vec
}

pub struct ImageSlice {
    width: u32,
    height: u32,
    rgba_vec: Vec<u8>,
}

impl ImageSlice {
    pub fn get_handle(self) -> Handle {
        Handle::from_rgba(self.width, self.height, self.rgba_vec)
    }
}