mod app;

use iced:: Size;
use crate::app::App;

fn  main() -> iced::Result {
    iced::application("My Image viewer", App::update, App::view)
        .theme(App::theme)
        .window_size(Size::new(500., 300.))
        .run_with(App::new)
}