mod file_loader;

use iced::widget::image::Handle;
use iced::widget::{button, column, container, image, row, slider, text};
use iced::{ContentFit, Element, Font, Length, Padding, Task};
use std::collections::HashMap;

use rfd::FileDialog;
use std::env;

use crate::app::file_loader::{load_image, load_images, open_folder, ImageSlice};
use iced::keyboard::key::Named::{ArrowDown, ArrowUp};
use iced::keyboard::Key;
use iced::Fill;
use iced_aw::style::selection_list::primary;
use iced_aw::SelectionList;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    OpenFolder,
    OpenedFolder((Vec<String>, HashMap<String, PathBuf>)),
    LoadedImage(Vec<ImageSlice>),
    SliderChanged(u8),
    FileSelected(usize, String),
    KeyPressed(Key),
}

pub struct App {
    image_handle: Option<Handle>,
    show_container: bool,
    image_name: String,
    slice_buffer: Vec<ImageSlice>,
    file_names: Vec<String>,
    current_slice: u8,
    selected_idx: Option<usize>,
    path_map: HashMap<String, PathBuf>,
}

impl App {
    pub(crate) fn new() -> (Self, Task<Message>) {
        let mut image_handle = None;
        let mut show_container = false;
        let mut image_name = String::from("No image loaded...");

        // For opening files via "open with" on Linux and Windows
        if env::args().len() > 1 {
            let args: Vec<String> = env::args().collect();
            let path_str = args.get(1).unwrap();

            let path = PathBuf::from(path_str);

            image_name = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
                .split("/")
                .last()
                .unwrap()
                .to_string();

            image_handle = Some(load_image(path));
            show_container = true;
        }
        (
            Self {
                image_handle,
                show_container,
                image_name,
                file_names: Vec::new(),
                slice_buffer: Vec::new(),
                current_slice: 0,
                selected_idx: None,
                path_map: HashMap::new(),
            },
            Task::none(),
        )
    }

    pub(crate) fn theme(&self) -> iced::Theme {
        iced::Theme::CatppuccinMocha
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFolder => {
                self.show_container = true;
                let files = FileDialog::new()
                    .add_filter("text", &["", ".dcm", ".aim"])
                    .pick_folder();

                if let Some(path_buf) = files {
                    return Task::perform(async { open_folder(path_buf) }, Message::OpenedFolder);
                }
            }
            Message::OpenedFolder((series_names, path_map)) => {
                self.file_names = series_names;
                self.path_map = path_map;
                self.selected_idx = Some(0);
                let path_buf = self.path_map.get(&self.file_names[0]).unwrap().clone();
                return Task::perform(async { load_images(path_buf) }, Message::LoadedImage);
            }
            Message::LoadedImage(slice_buffer) => {
                self.slice_buffer = slice_buffer;
                if self.slice_buffer.len() > 0 {
                    if self.slice_buffer[0].rgba_vec.len() > 0 {
                        self.image_handle = Some(self.slice_buffer[0].clone().get_handle());
                        self.show_container = true;
                    } else {
                        self.image_handle = None;
                        self.show_container = false;
                    }
                } else {
                    self.image_handle = None;
                    self.show_container = false;
                }
                let image_path = self.slice_buffer[0].file_name.clone();
                let image_path = image_path.split("/").last().unwrap();
                self.image_name = image_path.to_string();
            }
            Message::SliderChanged(current_slice) => {
                let index = current_slice as usize;
                self.image_handle = Some(self.slice_buffer[index].clone().get_handle());
                self.current_slice = current_slice;

                let image_path = self.slice_buffer[index].file_name.clone();
                let image_path = image_path.split("/").last().unwrap();
                self.image_name = image_path.to_string();
            }
            Message::FileSelected(index, file_name) => {
                let path_buf = self.path_map.get(&file_name).unwrap().clone();
                self.current_slice = 0;
                self.selected_idx = Some(index);
                return Task::perform(async { load_images(path_buf) }, Message::LoadedImage);
            }
            Message::KeyPressed(key) => {
                if let Some(selected_idx) = self.selected_idx {
                    let mut new_idx: Option<usize> = None;
                    if key == Key::Named(ArrowDown) {
                        if selected_idx < self.file_names.len() - 1 {
                            new_idx = Some(selected_idx + 1);
                        }
                    } else if key == Key::Named(ArrowUp) {
                        if selected_idx > 0 {
                            new_idx = Some(selected_idx - 1);
                        }
                    }
                    if let Some(new_idx) = new_idx {
                        let filename = self.file_names[new_idx].clone();
                        self.selected_idx = Some(new_idx);
                        let path_buf = self.path_map.get(&filename).unwrap().clone();
                        return Task::perform(
                            async { load_images(path_buf) },
                            Message::LoadedImage,
                        );
                    }
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        container(row!(
            column!(image_view(self))
                .padding(5)
                .width(Length::FillPortion(3)),
            column!(menu_view(self))
                .padding(5)
                .width(Length::FillPortion(2)),
        ))
        .padding(15)
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }
}

fn image_view(state: &App) -> Element<Message> {
    if state.show_container {
        match &state.image_handle {
            Some(img_handle) => container(
                row!(column![
                    container(text(&state.image_name)).padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 10.0,
                        left: 0.0
                    }),
                    container(
                        image(img_handle.clone()).content_fit(ContentFit::ScaleDown)
                    )
                    .center(Fill)
                    .width(Fill)
                    .height(Fill),
                    container(
                        row!(
                            slider(
                                1..=(state.slice_buffer.len() as u8 - 1),
                                state.current_slice,
                                Message::SliderChanged
                            )
                            .width(Length::FillPortion(8)),
                            container(text(format!("{}/{}", state.current_slice, state.slice_buffer.len())))
                            .align_right(Fill)
                            .padding(Padding {top: 0.0, right: 0.0, bottom: 10.0, left: 5.0})
                            .width(Length::FillPortion(2)),
                        )
                    ),
                ],)
                .padding(5),
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

    container(column![
        button("Load").on_press(Message::OpenFolder),
        container(selection_list).padding(15),
    ])
    .height(Fill)
    .into()
}
