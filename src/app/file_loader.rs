use std::cmp::Ordering;
use std::collections::HashMap;

use std::fs;
use std::path::PathBuf;
use dicom::core::Tag;
use dicom::object::{open_file};
use dicom::pixeldata::image::{GenericImageView};
use dicom::pixeldata::{PixelDecoder};
use iced::widget::image::Handle;
use nifti::{InMemNiftiVolume, IntoNdArray, NiftiObject, NiftiVolume, ReaderOptions, Sliceable};
use nifti::volume::SliceView;

pub fn open_folder(path_buf: PathBuf) -> (Vec<String>, HashMap<String, PathBuf>) {

    let mut path_map:HashMap<String, PathBuf> = HashMap::new();
    let mut series_names:Vec<String> = Vec::new();

    if path_buf.is_dir() {
        for entry in fs::read_dir(path_buf).unwrap() {
            match entry {
                Ok(path) => {
                    // ignore hidden files like .DS_Store on mac
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

    nifi_to_rgba(volume)

    // let axis = get_correct_axis(volume);

    // let dims = volume.dim();
    // let (biggest_idx, sec_biggest_idx) = get_two_biggest_dims_with_idx(dims);
    // let width = dims[biggest_idx] as u32;
    // let height = dims[sec_biggest_idx] as u32;

    // let mut img_vec:Vec<ImageSlice> = vec![];

    // for i in 0..20 {
        // axis scheint hier komplett egal zu sein...
    //     let slice = volume.get_slice(axis, i).unwrap();
    //     let rgba_vec = convert_slice_to_rgba(slice);
    //     let location = Some(i as f32);

    //     img_vec.push(ImageSlice {
    //        file_name: file_name.clone(),
    //        width,
    //        height,
    //        rgba_vec,
    //        location,
    //     });
    //}
    // img_vec
}

fn convert_slice_to_rgba(slice: SliceView<&InMemNiftiVolume>) -> Vec<u8> {
    let data:Vec<u8> = slice.into_ndarray().unwrap().into_raw_vec();
    let mut rgba_vec = Vec::<u8>::with_capacity(data.len() * 4);
    for value in data {
        for _ in 0..3 {
            rgba_vec.push(value);
        }
        rgba_vec.push(255);
    }
    rgba_vec
}

fn nifi_to_rgba(volume: InMemNiftiVolume) -> Vec<ImageSlice> {
    let dims = volume.dim();
    let height = dims[0] as usize;
    let width = dims[1] as usize;
    let num_slices = dims[2] as usize;

    let num_slice_pixels = height * width;

    let raw_data = volume.into_raw_data();

    let mut img_vec:Vec<ImageSlice> = vec![];

    for slice_idx in 0..num_slices {
        let start_idx = slice_idx * num_slice_pixels;
        let end_idx = start_idx + num_slice_pixels;

        let mut slice = Vec::<u8>::with_capacity((num_slice_pixels * 4) as usize);

        for i in start_idx..end_idx {
            let pixel_val = raw_data[i as usize];
            slice.push(pixel_val);
            slice.push(pixel_val);
            slice.push(pixel_val);
            slice.push(255);
        }
        img_vec.push(ImageSlice {
            file_name: "file_name".to_string(),
            width: width as u32,
            height: height as u32,
            rgba_vec : slice,
            location: Some(slice_idx as f32),
        });
    }
    img_vec
}


fn get_correct_axis(volume: &InMemNiftiVolume) -> u16 {
    let volume_dims = volume.dim();
    if volume_dims.len() == 2 {
        2
    } else {
        let (biggest_idx, sec_biggest_idx) = get_two_biggest_dims_with_idx(volume_dims);
        if (biggest_idx == 0 && sec_biggest_idx == 1) || (biggest_idx == 1 && sec_biggest_idx == 0) {
            1
        } else if (biggest_idx == 1 && sec_biggest_idx == 2) || (biggest_idx == 2 && sec_biggest_idx == 1) {
            1
        } else if (biggest_idx == 0 && sec_biggest_idx == 2) || (biggest_idx == 2 && sec_biggest_idx == 0) {
            1
        } else {
            2
        }
    }
}


fn get_two_biggest_dims_with_idx(arr: &[u16]) -> (usize,usize) {
    let mut bigest:u16 = 0;
    let mut bigest_idx:usize = 0;
    let mut sec_bigest:u16 = 0;
    let mut sec_bigest_idx:usize = 0;

    for i in 0..arr.len() {
        if arr[i] > bigest {
            if bigest != 0 {
                sec_bigest = bigest;
                sec_bigest_idx = bigest_idx
            }
            bigest = arr[i];
            bigest_idx = i;
            continue;
        }
        if arr[i] > sec_bigest {
            sec_bigest = arr[i];
            sec_bigest_idx = i;
            continue;
        }
    }
    (bigest_idx, sec_bigest_idx)
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

