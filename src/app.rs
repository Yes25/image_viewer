use iced::{Element, Task};
use iced::widget::image::Handle;
use iced::widget::{button, container, image, row};

use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone)]
pub enum Message {
    Load,
    Loaded(Vec<u8>),
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

            Task::none()
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
                        let mut file = File::open("/Users/jesse/Code/rust_proj/test_img/ferris.png").await.unwrap();
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).await.unwrap();
                        buffer

                    },
                    Message::Loaded,
                );
            }
            Message::Loaded(data) => self.image_handle = Some(Handle::from_bytes(data)),
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

