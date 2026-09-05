mod connection;
mod gpio;
mod log;

use iced::{
    Background, Element, Length,
    widget::{button, column, container, pane_grid, text},
};

use crate::{
    app::{App, Message, PaneKind},
    session::PinInfo,
    theme::{WINDOW_BG, danger_button, neutral_button},
};

pub(super) const CONTROL_TEXT_SIZE: f32 = 13.0;

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let top = app.connection_controls();
    let body = pane_grid(&app.panes, |_, pane, _| {
        pane_grid::Content::new(match pane {
            PaneKind::Pins => app.pin_panel(),
            PaneKind::Log => app.log_panel(),
        })
    })
    .spacing(8)
    .min_size(400)
    .on_resize(8, Message::PaneResized)
    .height(Length::Fill);

    let mut content = column![top, body].spacing(8).padding(8);
    if let Some(error) = &app.error {
        content = content.push(
            container(text(format!("Error: {error}")).size(13))
                .padding([6, 10])
                .style(container::danger),
        );
    }
    container(content)
        .height(Length::Fill)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(WINDOW_BG)),
            ..Default::default()
        })
        .into()
}

pub(super) fn native_button<'a>(
    label: impl text::IntoFragment<'a>,
) -> iced::widget::Button<'a, Message> {
    button(
        text(label)
            .size(CONTROL_TEXT_SIZE)
            .wrapping(text::Wrapping::None),
    )
    .padding([5, 10])
    .style(neutral_button)
}

pub(super) fn danger_native_button<'a>(
    label: impl text::IntoFragment<'a>,
) -> iced::widget::Button<'a, Message> {
    native_button(label).style(danger_button)
}

pub(super) fn pin_display(pin: &PinInfo) -> String {
    gpio::pin_display(pin)
}
