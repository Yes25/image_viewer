mod app;

use crate::app::App;
use iced::Size;

fn main() -> iced::Result {
    iced::application("My Image viewer", App::update, App::view)
        .theme(App::theme)
        .window_size(Size::new(700., 500.))
        .run_with(App::new)
}
