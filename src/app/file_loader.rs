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