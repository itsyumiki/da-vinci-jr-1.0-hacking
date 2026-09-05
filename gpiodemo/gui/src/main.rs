mod app;
mod io;
mod serial_log;
mod session;
mod theme;
mod view;

use app::App;
use iced::Size;
use theme::app_theme;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("GPIO Controller")
        .subscription(App::subscription)
        .theme(app_theme())
        .window(iced::window::Settings {
            size: Size::new(1280.0, 820.0),
            min_size: Some(Size::new(900.0, 640.0)),
            ..Default::default()
        })
        .run()
}
