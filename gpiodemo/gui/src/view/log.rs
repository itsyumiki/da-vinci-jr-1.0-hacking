use iced::{
    Element, Length,
    widget::{checkbox, column, container, row, scrollable, text, text_editor, text_input},
};

use super::{CONTROL_TEXT_SIZE, native_button};
use crate::app::{App, Message};

impl App {
    pub(super) fn log_panel(&self) -> Element<'_, Message> {
        let log = scrollable(
            text_editor(self.log.content())
                .font(iced::Font::MONOSPACE)
                .size(12)
                .padding(8)
                .on_action(Message::LogAction),
        )
        .id(self.log_scroll.clone())
        .height(Length::Fill)
        .width(Length::Fill);
        let options = column![
            text("Serial Log").size(18),
            row![
                native_button("Clear log").on_press(Message::ClearLog),
                checkbox(self.log.show_timestamps())
                    .label("Timestamps")
                    .size(16)
                    .spacing(5)
                    .text_size(CONTROL_TEXT_SIZE)
                    .on_toggle(Message::ShowTimestamps),
                checkbox(self.autoscroll)
                    .label("Auto-scroll")
                    .size(16)
                    .spacing(5)
                    .text_size(CONTROL_TEXT_SIZE)
                    .on_toggle(Message::Autoscroll),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .wrap()
            .vertical_spacing(4),
        ]
        .spacing(6);
        let command = row![
            text_input("Enter a command…", &self.raw_input)
                .id(self.raw_input_id.clone())
                .font(iced::Font::MONOSPACE)
                .on_input(Message::RawChanged)
                .on_submit(Message::RawSend),
            native_button("Send command").on_press(Message::RawSend),
        ]
        .spacing(5);

        container(column![options, log, command].spacing(8))
            .padding(12)
            .style(container::bordered_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
