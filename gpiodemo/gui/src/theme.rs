use iced::{
    Background, Border, Color, Theme,
    widget::{button, container},
};

pub(super) const WINDOW_BG: Color = Color::from_rgb8(0x24, 0x24, 0x24);
const PANEL_BG: Color = Color::from_rgb8(0x2B, 0x2B, 0x2B);
const RAISED_BG: Color = Color::from_rgb8(0x3A, 0x3A, 0x3A);
const RAISED_HOVER: Color = Color::from_rgb8(0x46, 0x46, 0x46);
const INPUT_BG: Color = Color::from_rgb8(0x34, 0x36, 0x38);
const UI_BORDER: Color = Color::from_rgb8(0x56, 0x5B, 0x5E);
pub(super) const UI_TEXT: Color = Color::from_rgb8(0xDC, 0xE4, 0xEE);
const MUTED: Color = Color::from_rgb8(0xB0, 0xB0, 0xB0);
pub(super) const HIGH_BG: Color = Color::from_rgb8(0x3D, 0xDC, 0x97);
pub(super) const LOW_BG: Color = Color::from_rgb8(0x4A, 0x4A, 0x4A);
pub(super) const UNSET_BG: Color = Color::from_rgb8(0x38, 0x38, 0x38);
const DANGER: Color = Color::from_rgb8(0xE0, 0x6C, 0x75);

pub(super) fn app_theme() -> Theme {
    Theme::custom(
        "GPIO Controller",
        iced::theme::Palette {
            background: WINDOW_BG,
            text: UI_TEXT,
            primary: Color::from_rgb8(0x3B, 0x8E, 0xD0),
            success: HIGH_BG,
            warning: Color::from_rgb8(0xD6, 0xA8, 0x4B),
            danger: DANGER,
        },
    )
}
pub(super) fn panel_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(PANEL_BG)),
        border: Border {
            color: UI_BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub(super) fn input_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(INPUT_BG)),
        text_color: Some(UI_TEXT),
        border: Border {
            color: UI_BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub(super) fn neutral_button(_: &Theme, status: button::Status) -> button::Style {
    let (background, text_color) = match status {
        button::Status::Hovered => (RAISED_HOVER, UI_TEXT),
        button::Status::Disabled => (RAISED_BG, MUTED),
        button::Status::Active | button::Status::Pressed => (RAISED_BG, UI_TEXT),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: UI_BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub(super) fn selected_tab_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = neutral_button(theme, status);
    style.background = Some(Background::Color(if status == button::Status::Hovered {
        Color::from_rgb8(0x50, 0x50, 0x50)
    } else {
        Color::from_rgb8(0x46, 0x46, 0x46)
    }));
    style
}

pub(super) fn danger_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = neutral_button(theme, status);
    style.text_color = if status == button::Status::Disabled {
        MUTED
    } else {
        DANGER
    };
    style
}
