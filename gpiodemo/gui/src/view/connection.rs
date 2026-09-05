use iced::{
    Element, Length,
    alignment::Horizontal,
    widget::{column, container, pick_list, responsive, row, text},
};

use super::{CONTROL_TEXT_SIZE, danger_native_button, native_button};
use crate::{
    app::{App, Message},
    theme::panel_style,
};

const CONNECTION_ACTIONS_INLINE_MIN: f32 = 1_050.0;

impl App {
    pub(super) fn connection_controls(&self) -> Element<'_, Message> {
        let content = responsive(|size| self.connection_content(size.width))
            .height(Length::Shrink)
            .width(Length::Fill);

        container(content)
            .padding(8)
            .width(Length::Fill)
            .style(panel_style)
            .into()
    }

    fn connection_content(&self, available_width: f32) -> Element<'_, Message> {
        let ports = pick_list(
            self.ports.as_slice(),
            self.selected_port.as_ref(),
            Message::PortSelected,
        )
        .placeholder("Serial port")
        .text_size(CONTROL_TEXT_SIZE)
        .padding([5, 8])
        .width(Length::Fill);

        let routes = pick_list(
            self.routes.as_slice(),
            Some(&self.selected_route),
            Message::RouteSelected,
        )
        .text_size(CONTROL_TEXT_SIZE)
        .padding([5, 8]);

        let connection_button = if self.connected_port.is_some() {
            native_button("Disconnect").on_press(Message::Disconnect)
        } else {
            native_button("Connect")
                .on_press_maybe(self.selected_port.as_ref().map(|_| Message::Connect))
        };

        let connection = row![
            text("Connection").size(18),
            ports,
            native_button("Refresh").on_press(Message::RefreshPorts),
            connection_button,
            text("Route").size(12),
            routes,
            text(&self.device_status).size(13),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill);

        let actions = row![
            native_button("Pin map").on_press(Message::PinMap),
            native_button("Handshake").on_press(Message::Handshake),
            native_button("Status").on_press(Message::Status),
            native_button("Version").on_press(Message::Version),
            native_button("Help").on_press(Message::Help),
            danger_native_button("Reset device").on_press(Message::Reboot),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let connection: Element<'_, Message> = if available_width >= CONNECTION_ACTIONS_INLINE_MIN {
            row![connection, actions]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .width(Length::Fill)
                .into()
        } else {
            column![
                connection,
                container(actions)
                    .width(Length::Fill)
                    .align_x(Horizontal::Right)
            ]
            .spacing(6)
            .into()
        };

        let content: Element<'_, Message> = if self.confirm_reboot {
            column![
                connection,
                row![
                    text("Reset device and drop the connection?").size(12),
                    danger_native_button("Reset device").on_press(Message::RebootConfirm),
                    native_button("Cancel").on_press(Message::RebootCancel),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(6)
            .into()
        } else {
            connection
        };

        content
    }
}
