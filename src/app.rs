mod file_loader;

use iced::widget::image::Handle;
use iced::widget::{button, column, container, image, row, slider, text, vertical_space};
use iced::{Element, Font, Length, Padding, Task};

use rfd::FileDialog;
use std::env;

use std::path::PathBuf;
use iced::Fill;
use iced_aw::SelectionList;
use iced_aw::style::selection_list::primary;
use crate::app::file_loader::{load_image, load_images, ImageSlice};

#[derive(Debug, Clone)]
pub enum Message {
    Load,
    Loaded(Vec<ImageSlice>),
    SliderChanged(u8),
    FileSelected(usize, String),
}

pub struct App {
    image_handle: Option<Handle>,
    show_container: bool,
    image_name: String,
    slice_buffer: Vec<ImageSlice>,
    file_names: Vec<String>,
    current_slice: u8,
    selected_idx: Option<usize>,
}

impl App {
    pub(crate) fn new() -> (Self, Task<Message>) {
        let mut image_handle = None;
        let mut show_container = false;
        let mut image_name = String::from("No image loaded...");

        if env::args().len() > 1 {
            let args: Vec<String> = env::args().collect();
            let path_str = args.get(1).unwrap();

            let path = PathBuf::from(path_str);

            image_name = path.file_name().unwrap().to_str().unwrap().to_owned();
            image_handle = Some(load_image(path));
            show_container = true;
        }
        (
            Self {
                image_handle,
                show_container,
                image_name,
                // file_names: Vec::new(),
                file_names: vec![String::from("file_1"), String::from("file_2"), String::from("file_3"), String::from("file_4")],
                slice_buffer: Vec::new(),
                current_slice: 0,
                selected_idx: None,
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
                    .pick_folder();


                if let Some(path_buf) = files {
                    return Task::perform(
                        async {
                            load_images(path_buf)
                        },
                        Message::Loaded,
                    );
                }
            }
            Message::Loaded(slice_buffer) => {
                self.slice_buffer = slice_buffer;
                self.image_handle = Some(self.slice_buffer[0].clone().get_handle());
                self.image_name = self.slice_buffer[0].file_name.clone();
            },
            Message::SliderChanged(current_slice) => {
                let index = current_slice as usize;
                self.image_handle = Some(self.slice_buffer[index].clone().get_handle());
                self.image_name = self.slice_buffer[index].file_name.clone();
                self.current_slice = current_slice;
            }
            Message::FileSelected(index, file_name) => {
                println!("Idx: {index} ::: file_name: {file_name}");
            }
        }
        Task::none()
    }


    pub fn view(&self) -> Element<Message> {
        container(
            row!(
                column!(
                    image_view(self)
                ).padding(5)
                .width(Length::FillPortion(3)),
                column!(
                    menu_view(self)
                )
                .padding(5)
                .width(Length::FillPortion(2)),
            )
        )
            .padding(15)
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }
}

fn image_view(state: &App) -> Element<Message> {

    if state.show_container {
        match &state.image_handle {
            Some(img_handle) => container(row!(
                column![
                    container(text(&state.image_name)).padding(Padding {top: 0.0 ,right: 0.0 ,bottom: 10.0, left: 0.0 }),
                    image(img_handle.clone()),
                    vertical_space(),
                    container( slider(1..=(state.slice_buffer.len() as u8 - 1), state.current_slice, Message::SliderChanged) ),
                ],
                ).padding(5)
            )
                .width(Fill)
                .into(),
            None => container("Loading...").width(Fill).into(),
        }
    } else {
        container("").width(Fill).into()
    }
}

fn menu_view(state: &App) -> Element<Message> {
    let selection_list = SelectionList::new_with(
        &state.file_names[..],
        Message::FileSelected,
        13.0,
        5.0,
        primary,
        state.selected_idx,
        Font::default(),
    )
        .width(Fill)
        .height(Fill);

    container(
        column![
            button("Load").on_press(Message::Load),
            container(selection_list).padding(15),
            ]
    )
        .height(Fill)
        .into()

}

