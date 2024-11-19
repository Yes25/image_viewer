mod file_loader;

use iced::widget::image::Handle;
use iced::widget::{button, container, image, row};
use iced::{Element, Task};

use rfd::FileDialog;
use std::env;
use std::path::PathBuf;

use crate::app::file_loader::load_image;

#[derive(Debug, Clone)]
pub enum Message {
    Load,
    Loaded(Handle),
}

pub struct App {
    image_handle: Option<Handle>,
    show_container: bool,
}

impl App {
    pub(crate) fn new() -> (Self, Task<Message>) {
        let mut image_handle = None;
        let mut show_container = false;

        if env::args().len() > 1 {
            let args: Vec<String> = env::args().collect();
            let path_str = args.get(1).unwrap();

            let path = PathBuf::from(path_str);

            image_handle = Some(load_image(path));
            show_container = true;

        }
        (
            Self {
                image_handle,
                show_container,
            },
            Task::none(),
        )
    }

    pub(crate) fn theme(&self) -> iced::Theme {
        iced::Theme::CatppuccinMocha
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Load => {
                self.show_container = true;
                let files = FileDialog::new()
                    .add_filter("text", &["", ".dcm", ".aim"])
                    .pick_file();

                if let Some(path_buf) = files {
                    return Task::perform(
                        async {
                            load_image(path_buf)
                        },
                        Message::Loaded,
                    );
                }
            }
            Message::Loaded(image_handle) => {
                self.image_handle = Some(image_handle)
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        container(
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
            .padding(20),
        )
        .into()
    }
}
