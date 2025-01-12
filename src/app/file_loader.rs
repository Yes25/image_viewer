use std::cmp::Ordering;
use std::collections::HashMap;

use std::fs;
use std::path::PathBuf;
use dicom::core::Tag;
use dicom::object::{open_file};
use dicom::pixeldata::image::{GenericImageView};
use dicom::pixeldata::{PixelDecoder};
use iced::widget::image::Handle;
use nifti::{InMemNiftiVolume, NiftiObject, NiftiVolume, ReaderOptions};

pub fn open_folder(path_buf: PathBuf) -> (Vec<String>, HashMap<String, PathBuf>) {

    let mut path_map:HashMap<String, PathBuf> = HashMap::new();
    let mut series_names:Vec<String> = Vec::new();

    if path_buf.is_dir() {
        for entry in fs::read_dir(path_buf).unwrap() {
            match entry {
                Ok(path) => {
                    // ignore hidden files like .DS_Store on Mac
                    if ! path.file_name().to_os_string().to_str().unwrap().starts_with(".") {
                        let path = path.path();

                        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
                        path_map.insert(file_name.clone(), path);
                        series_names.push(file_name);
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
        match path.extension() {
            Some(extension) => {
                if extension == "gz" {
                    image_vec = load_nifti_file(path);
                }
            }
            None => {
                image_vec.push(load_dicom_file(path));
            }
        }
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

fn load_dicom_file(path: PathBuf) -> ImageSlice{

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

    ImageSlice {
        file_name,
        width,
        height,
        rgba_vec,
        location,
    }
}

fn load_nifti_file(path: PathBuf) -> Vec<ImageSlice> {
    let file_name = path.to_str().unwrap().to_owned();

    let obj = ReaderOptions::new().read_file(path).unwrap();
    let volume = obj.volume().to_owned();

    nifi_to_rgba(volume, file_name)
}

fn nifi_to_rgba(volume: InMemNiftiVolume, file_name: String) -> Vec<ImageSlice> {
    let dims = volume.dim();
    let height = dims[0] as usize;
    let width = dims[1] as usize;
    let num_slices = dims[2] as usize;

    let num_slice_pixels = height * width;

    // let raw_data = volume.into_raw_data();
    let raw_data = volume.raw_data();
    let (min, max) = get_min_max(raw_data);

    let mut img_vec:Vec<ImageSlice> = vec![];

    for slice_idx in 0..num_slices {
        let start_idx = slice_idx * num_slice_pixels * 2;
        let end_idx = start_idx + num_slice_pixels * 2;

        let mut slice = Vec::<u8>::with_capacity(num_slice_pixels * 4);

        for i in (start_idx..end_idx).step_by(2) {
            // pixel vals are i16 -> 2 bytes -> conat two bytes to get value
            let i16_bytes = [raw_data[i], raw_data[i + 1]];
            // here little endian, could also be big? (is stored in volume struct)
            let pixel_val = scale_pix_val( min, max, u16::from_le_bytes(i16_bytes));

            slice.push(pixel_val); // r
            slice.push(pixel_val); // g
            slice.push(pixel_val); // b
            slice.push(255); // hue
        }

        img_vec.push(ImageSlice {
            file_name: file_name.clone(),
            width: width as u32,
            height: height as u32,
            rgba_vec : slice,
            location: Some(slice_idx as f32),
        });
    }
    img_vec
}

fn get_min_max(data: &[u8]) -> (u16, u16){
    let mut max:u16 = u16::MIN;
    let mut min:u16 = u16::MAX;
    for i in (0..data.len()).step_by(2) {
        let val = u16::from_le_bytes([data[i], data[i + 1]]);
        if val > max {max = val;}
        if val < min {min = val;}
    }
    (min, max)
}

fn scale_pix_val(min: u16, max: u16, pix_val: u16) -> u8{
    (( (pix_val - min) as f32 / max as f32) * 255.) as u8
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

