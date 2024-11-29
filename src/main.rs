mod app;

use crate::app::{App, Message};
use iced::{keyboard, Size, Subscription};

fn main() -> iced::Result {
    iced::application("My Image viewer", App::update, App::view)
        .theme(App::theme)
        .window_size(Size::new(770., 550.))
        .subscription(keyboard_subscription)
        .run_with(App::new)
}

fn keyboard_subscription(_app: &App) -> Subscription<Message> {
    keyboard::on_key_press( |key_pressed, _modifiers| Option::from(Message::KeyPressed(key_pressed)) )
}
