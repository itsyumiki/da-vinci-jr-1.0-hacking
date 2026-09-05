use da_vinci_protocol::{Level, PinCapabilities};
use iced::{
    Background, Border, Element, Length,
    alignment::{Horizontal, Vertical},
    widget::{checkbox, column, container, pick_list, responsive, row, scrollable, text},
};

use super::{CONTROL_TEXT_SIZE, danger_native_button, native_button};
use crate::{
    app::{App, BankGroup, MODES, Message},
    session::{BankKey, ListenerState, Mode, PinKey, PinState},
    theme::{HIGH_BG, LOW_BG, UI_TEXT, UNSET_BG, input_style, panel_style, selected_tab_button},
};

const ROW_HEIGHT: f32 = 34.0;
const PIN_CONTROL_TEXT_SIZE: f32 = 12.0;
const PIN_NAME_SHARE: u16 = 5;
const PIN_MODE_SHARE: u16 = 7;
const PIN_STATUS_SHARE: u16 = 3;
const PIN_RW_SHARE: u16 = 7;
const PIN_LISTEN_SHARE: u16 = 5;
const PIN_TABLE_TWO_COLUMN_MIN: f32 = 800.0;
const CELL_GAP: f32 = 4.0;

impl App {
    pub(super) fn pin_panel(&self) -> Element<'_, Message> {
        let index = self.bank_group;
        let mut tabs =
            row![native_button("‹").on_press_maybe((index > 0).then_some(Message::PreviousGroup))]
                .spacing(4)
                .align_y(iced::Alignment::Center);
        for (group_index, group) in self.bank_groups.iter().enumerate() {
            let tab_button = if group_index == index {
                native_button(&group.label).style(selected_tab_button)
            } else {
                native_button(&group.label)
            };
            tabs = tabs.push(tab_button.on_press(Message::GroupSelected(group_index)));
        }
        tabs = tabs
            .push(native_button("›").on_press_maybe(
                (index + 1 < self.bank_groups.len()).then_some(Message::NextGroup),
            ));
        let tabs = tabs.wrap().vertical_spacing(4);

        let bulk_modes = self.bulk_modes();
        let has_input = bulk_modes.iter().any(|mode| mode.is_input());
        let has_output = bulk_modes.contains(&Mode::Output);
        let selected_bulk_mode = bulk_modes
            .contains(&self.bulk_mode)
            .then_some(self.bulk_mode);
        let bulk_scope = row![
            text("Scope").size(12),
            pick_list(
                self.bulk_scopes.as_slice(),
                Some(&self.bulk_scope),
                Message::BulkScopeSelected
            )
            .text_size(PIN_CONTROL_TEXT_SIZE)
            .padding([5, 8]),
            text("Mode").size(12),
            pick_list(bulk_modes, selected_bulk_mode, Message::BulkModeSelected)
                .text_size(PIN_CONTROL_TEXT_SIZE)
                .padding([5, 8]),
            checkbox(self.overwrite)
                .label("Overwrite")
                .size(16)
                .spacing(5)
                .text_size(CONTROL_TEXT_SIZE)
                .on_toggle(Message::OverwriteChanged),
            native_button("Apply mode")
                .on_press_maybe(selected_bulk_mode.map(|_| Message::ApplyBulkMode)),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .wrap()
        .vertical_spacing(4);

        let bulk_actions: Element<'_, Message> = if let Some((target, level)) = &self.confirm_set {
            let level_name = match level {
                Level::High => "HIGH",
                Level::Low => "LOW",
            };
            row![
                text(format!("Set every output in {target} {level_name}?")),
                danger_native_button(format!("Set {level_name}")).on_press(Message::BulkSetConfirm),
                native_button("Cancel").on_press(Message::BulkSetCancel),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .wrap()
            .vertical_spacing(4)
            .into()
        } else {
            row![
                native_button("Read").on_press_maybe(has_input.then_some(Message::BulkRead)),
                native_button("Listen")
                    .on_press_maybe(has_input.then_some(Message::BulkListen(true))),
                native_button("Stop listening")
                    .on_press_maybe(has_input.then_some(Message::BulkListen(false))),
                native_button("Set HIGH")
                    .on_press_maybe(has_output.then_some(Message::BulkSet(Level::High))),
                native_button("Set LOW")
                    .on_press_maybe(has_output.then_some(Message::BulkSet(Level::Low))),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .wrap()
            .vertical_spacing(4)
            .into()
        };
        let bulk = column![bulk_scope, bulk_actions].spacing(4);

        let table = self.bank_groups.get(index).map_or_else(
            || text("Waiting for pin map…").into(),
            |group| self.group_table(group),
        );
        let content = column![tabs, bulk, table].spacing(6);

        container(content)
            .padding(8)
            .style(panel_style)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn group_table(&self, group: &BankGroup) -> Element<'_, Message> {
        match group.banks.as_slice() {
            [] => text("No discovered banks").into(),
            [bank] => self.single_bank_table(*bank),
            [left, right] => {
                let (left, right) = (*left, *right);
                responsive(move |size| {
                    responsive_pin_columns(
                        size.width,
                        self.bank_column(left, 0, None, true),
                        self.bank_column(right, 0, None, true),
                    )
                })
                .height(Length::Fill)
                .into()
            }
            banks => {
                let mut content = column![].spacing(8);
                for &bank in banks {
                    content = content.push(self.bank_column(bank, 0, None, true));
                }
                scrollable(content)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
            }
        }
    }

    fn single_bank_table(&self, bank: BankKey) -> Element<'_, Message> {
        let route = self.selected_route_key();
        let has_upper_half = self
            .session
            .pins(route)
            .any(|(_, info)| info.bank == bank && info.bit >= 16);
        if has_upper_half {
            responsive(move |size| {
                responsive_pin_columns(
                    size.width,
                    self.bank_column(bank, 0, Some(16), false),
                    self.bank_column(bank, 16, None, false),
                )
            })
            .height(Length::Fill)
            .into()
        } else {
            scrollable(self.bank_column(bank, 0, None, false))
                .height(Length::Fill)
                .width(Length::Fill)
                .into()
        }
    }

    fn bank_column(
        &self,
        bank: BankKey,
        start_bit: u8,
        end_bit: Option<u8>,
        show_bank_name: bool,
    ) -> iced::widget::Column<'_, Message> {
        let mut column = column![].spacing(2);
        if show_bank_name && let Some(info) = self.session.bank_info(bank) {
            column = column.push(text(info.token.clone()).size(14));
        }
        column = column.push(pin_header());
        for (pin, info) in self.session.pins(self.selected_route_key()) {
            if info.bank != bank
                || info.bit < start_bit
                || end_bit.is_some_and(|end| info.bit >= end)
            {
                continue;
            }
            column = column.push(self.pin_row(pin));
        }
        column
    }

    fn pin_row(&self, pin: PinKey) -> Element<'_, Message> {
        let Some(info) = self.session.pin_info(pin) else {
            return text("Unknown pin").into();
        };
        let name = pin_cell(text(info.to_string()).size(12), PIN_NAME_SHARE);
        if !info.capabilities.available() {
            return row![
                name,
                pin_cell(text("RESERVED").size(11), PIN_MODE_SHARE),
                level_box(None, false),
                pin_cell(text("System").size(11), PIN_RW_SHARE),
                pin_cell(text(""), PIN_LISTEN_SHARE),
            ]
            .spacing(CELL_GAP)
            .width(Length::Fill)
            .height(Length::Fixed(ROW_HEIGHT))
            .align_y(iced::Alignment::Center)
            .into();
        }

        let state = self.session.pin_state(pin).unwrap_or(PinState::UNSET);
        let mode: Element<'_, Message> = if state.target_mode.is_some() {
            container(
                text(
                    state
                        .mode
                        .map(|mode| mode.to_string())
                        .unwrap_or_else(|| "UNSET".into()),
                )
                .size(PIN_CONTROL_TEXT_SIZE)
                .wrapping(text::Wrapping::None),
            )
            .padding([5, 10])
            .width(Length::Fill)
            .style(input_style)
            .into()
        } else {
            pick_list(pin_modes(info.capabilities), state.mode, move |mode| {
                Message::ModeSelected(pin, mode)
            })
            .placeholder("UNSET")
            .text_size(PIN_CONTROL_TEXT_SIZE)
            .padding([5, 8])
            .width(Length::Fill)
            .into()
        };

        let rw: Element<'_, Message> =
            if state.mode.is_some_and(Mode::is_input) && info.capabilities.input() {
                native_button("Read")
                    .on_press_maybe((!state.value_pending).then_some(Message::Read(pin)))
                    .into()
            } else if state.mode == Some(Mode::Output) && info.capabilities.output() {
                let label = if state.level == Some(Level::High) {
                    "Write LOW"
                } else {
                    "Write HIGH"
                };
                native_button(label)
                    .on_press_maybe((!state.value_pending).then_some(Message::Write(pin)))
                    .into()
            } else {
                text("").into()
            };

        let listen: Element<'_, Message> = if state.mode.is_some_and(Mode::is_input)
            && info.capabilities.input()
        {
            let label = if matches!(state.listener, ListenerState::On | ListenerState::Disabling) {
                "Stop"
            } else {
                "Listen"
            };
            native_button(label)
                .on_press_maybe((!state.listener.is_pending()).then_some(Message::Listen(pin)))
                .into()
        } else {
            text("").into()
        };

        row![
            name,
            pin_cell(mode, PIN_MODE_SHARE),
            level_box(state.level, state.value_pending),
            pin_cell(rw, PIN_RW_SHARE),
            pin_cell(listen, PIN_LISTEN_SHARE),
        ]
        .spacing(CELL_GAP)
        .width(Length::Fill)
        .height(Length::Fixed(ROW_HEIGHT))
        .align_y(iced::Alignment::Center)
        .into()
    }
}

fn pin_modes(capabilities: PinCapabilities) -> &'static [Mode] {
    match (
        capabilities.input(),
        capabilities.pull_up(),
        capabilities.output(),
    ) {
        (false, _, false) => &[],
        (true, false, false) => &[Mode::Input],
        (true, true, false) => &[Mode::Input, Mode::InputPullup],
        (false, _, true) => &[Mode::Output],
        (true, false, true) => &[Mode::Input, Mode::Output],
        (true, true, true) => &MODES,
    }
}

fn pin_header<'a>() -> Element<'a, Message> {
    row![
        pin_cell(text("PIN").size(11), PIN_NAME_SHARE),
        pin_cell(text("MODE").size(11), PIN_MODE_SHARE),
        pin_cell(text("LEVEL").size(11), PIN_STATUS_SHARE),
        pin_cell(text("READ/WRITE").size(10), PIN_RW_SHARE),
        pin_cell(text("LISTEN/STOP").size(10), PIN_LISTEN_SHARE),
    ]
    .spacing(CELL_GAP)
    .width(Length::Fill)
    .into()
}

fn responsive_pin_columns<'a>(
    available_width: f32,
    left: iced::widget::Column<'a, Message>,
    right: iced::widget::Column<'a, Message>,
) -> Element<'a, Message> {
    let content: Element<'a, Message> = if available_width >= PIN_TABLE_TWO_COLUMN_MIN {
        row![
            left.width(Length::FillPortion(1)),
            right.width(Length::FillPortion(1)),
        ]
        .spacing(8)
        .width(Length::Fill)
        .into()
    } else {
        column![left.width(Length::Fill), right.width(Length::Fill)]
            .spacing(8)
            .width(Length::Fill)
            .into()
    };

    scrollable(content)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn pin_cell<'a>(content: impl Into<Element<'a, Message>>, share: u16) -> Element<'a, Message> {
    container(content)
        .width(Length::FillPortion(share))
        .height(Length::Fixed(ROW_HEIGHT))
        .align_x(Horizontal::Left)
        .align_y(Vertical::Center)
        .into()
}

fn level_box(level: Option<Level>, pending: bool) -> Element<'static, Message> {
    let label = if pending {
        "…"
    } else {
        match level {
            Some(Level::High) => "HIGH",
            Some(Level::Low) => "LOW",
            None => "—",
        }
    };
    let background = if pending {
        UNSET_BG
    } else {
        match level {
            Some(Level::High) => HIGH_BG,
            Some(Level::Low) => LOW_BG,
            None => UNSET_BG,
        }
    };
    container(text(label).size(11))
        .width(Length::FillPortion(PIN_STATUS_SHARE))
        .height(Length::Fixed(28.0))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .style(move |_| container::Style {
            background: Some(Background::Color(background)),
            text_color: Some(UI_TEXT),
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_modes_follow_discovered_capabilities() {
        assert_eq!(pin_modes(PinCapabilities::NONE), []);
        assert_eq!(pin_modes(PinCapabilities::INPUT), [Mode::Input]);
        assert_eq!(
            pin_modes(PinCapabilities::INPUT_PULLUP),
            [Mode::Input, Mode::InputPullup]
        );
        assert_eq!(pin_modes(PinCapabilities::OUTPUT), [Mode::Output]);
        assert_eq!(pin_modes(PinCapabilities::GPIO), MODES);
    }
}
