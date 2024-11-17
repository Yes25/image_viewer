use iced::widget::image::Handle;
use iced::widget::{button, container, image, row};
use iced::{Element, Task};

use dicom::object::open_file;
use dicom::pixeldata::image::GenericImageView;
use dicom::pixeldata::PixelDecoder;

#[derive(Debug, Clone)]
pub enum Message {
    Load,
    Loaded((u32, u32, Vec<u8>)),
}

pub struct App {
    image_handle: Option<Handle>,
    show_container: bool,
}

impl App {
    pub(crate) fn new() -> (Self, Task<Message>) {
        (
            Self {
                image_handle: None,
                show_container: false,
            },
            Task::none(),
        )
    }

    pub(crate) fn theme(&self) -> iced::Theme {
        iced::Theme::TokyoNightStorm
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Load => {
                self.show_container = true;
                return Task::perform(
                    async {
                        let obj =
                            open_file("/Users/jessekruse/Downloads/2_skull_ct/DICOM/I0").unwrap();

                        let decoded_pixel_data = obj.decode_pixel_data().unwrap();
                        let dyn_img = decoded_pixel_data.to_dynamic_image(0).unwrap();

                        let (width, hight) = dyn_img.dimensions() as (u32, u32);

                        let rgba_img = dyn_img.to_rgba8();
                        let rgba_vec = rgba_img.to_vec();

                        (width, hight, rgba_vec)
                    },
                    Message::Loaded,
                );
            }
            Message::Loaded((x, y, rgba_vec)) => {
                self.image_handle = Some(Handle::from_rgba(x, y, rgba_vec))
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        row!(
            button("Load").on_press(Message::Load),
            if self.show_container {
                match &self.image_handle {
                    Some(h) => container(image(h.clone())),
                    None => container("Loading..."),
                }
            } else {
                container("")
            }
        )
        .padding(20)
        .into()
    }
}