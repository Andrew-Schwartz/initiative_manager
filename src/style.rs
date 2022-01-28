use std::fmt::{self, Display};
use std::ops::Not;

use iced::{button, checkbox, container, pick_list, scrollable, slider, text_input};
use iced_aw::tabs;

macro_rules! from {
    (
        @priv $style:ident => $module:ident: dark = $dark:ident
    ) => {
        from! { @priv-final $style => $module: light = Default::default(), dark = dark::$dark.into() }
    };
    (
        @priv $style:ident => $module:ident: light = $light:ident, dark = $dark:ident
    ) => {
        from! { @priv-final $style => $module: light = Default::default(), dark = dark::$dark.into() }
    };
    (
        @priv $style:ident => $module:ident: dark = $dark:ident,light = $light:ident
    ) => {
        from! { @priv-final $style => $module: light = Default::default(), dark = dark::$dark.into() }
    };
    (
        @priv-final $style:ident => $module:ident: light = $light:expr, dark = $dark:expr
    ) => {
        impl From<$style> for Box<dyn $module::StyleSheet> {
            fn from(style: $style) -> Self {
                match style {
                    $style::Light => $light,
                    $style::Dark => $dark,
                }
            }
        }
    };
    (
        $style:ident =>
        $($module:ident: $($light_dark_token:tt = $light_dark:ident),*);* $(;)?
    ) => {
        $(
            from! { @priv $style => $module: $($light_dark_token = $light_dark),* }
        )*
    };
}

macro_rules! color {
    (rgb $r:literal $g:literal $b:literal) => {
        color!(rgba $r $g $b 0xFF)
    };
    (rgba $r:literal $g:literal $b:literal $a:literal) => {
        Color {
            r: $r as f32 / 255.0,
            g: $g as f32 / 255.0,
            b: $b as f32 / 255.0,
            a: $a as f32 / 255.0,
        }
    };
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Style {
    Light,
    Dark,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SettingsBarStyle {
    Light,
    Dark,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct InitiativeTableStyle {
    style: Style,
    alt: Option<bool>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InitiativeTableBorderStyle {
    Light,
    Dark,
}

impl Style {
    pub fn settings_bar(self) -> SettingsBarStyle {
        match self {
            Self::Light => SettingsBarStyle::Light,
            Self::Dark => SettingsBarStyle::Dark,
        }
    }

    pub fn initiative_table(self, n: usize) -> InitiativeTableStyle {
        InitiativeTableStyle {
            style: self,
            alt: (n != 0).then(|| n % 2 == 1),
        }
    }

    pub fn initiative_table_border(self) -> InitiativeTableBorderStyle {
        match self {
            Self::Light => InitiativeTableBorderStyle::Light,
            Self::Dark => InitiativeTableBorderStyle::Dark,
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::Dark
    }
}

impl Not for Style {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Style::Light => "Light",
            Style::Dark => "Dark",
        })
    }
}

from! { Style =>
    container: dark = Container;
    text_input: dark = TextInput;
    scrollable: dark = Scrollable;
    button: light = Button, dark = Button;
    pick_list: dark = PickList;
    checkbox: dark = Checkbox;
    slider: dark = Slider;
    tabs: dark = Tabs;
}

from! { SettingsBarStyle =>
    button: light = Button, dark = SettingsButton;
    container: dark = SettingsContainer;
}

from! { InitiativeTableBorderStyle =>
    container: dark = InitiativeTableBorder;
}

// from! { InitiativeTableStyle =>
//     button: dark = InitiativeTable
// }

// todo epic macro for this too :)
impl From<InitiativeTableStyle> for Box<dyn container::StyleSheet> {
    fn from(InitiativeTableStyle { style, alt }: InitiativeTableStyle) -> Self {
        match style {
            Style::Light => Default::default(),
            Style::Dark => dark::InitiativeTable(alt).into(),
        }
    }
}

impl From<InitiativeTableStyle> for Box<dyn button::StyleSheet> {
    fn from(InitiativeTableStyle { style, alt }: InitiativeTableStyle) -> Self {
        match style {
            Style::Light => Default::default(),
            Style::Dark => dark::InitiativeTable(alt).into(),
        }
    }
}

impl From<InitiativeTableStyle> for Box<dyn text_input::StyleSheet> {
    fn from(InitiativeTableStyle { style, alt }: InitiativeTableStyle) -> Self {
        match style {
            Style::Light => Default::default(),
            Style::Dark => dark::InitiativeTable(alt).into(),
        }
    }
}

mod light {
    use iced::{button, Color};

    pub struct Button;

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                // background: Color::from_rgb8(0xAD, 0xAD, 0xCD).into(),
                // border_radius: 4.0,
                // text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                ..Default::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                // text_color: Color::WHITE,
                ..self.active()
            }
        }

        fn pressed(&self) -> button::Style {
            button::Style {
                // border_width: 1.0,
                // border_color: [0.2, 0.2, 0.2].into(),
                ..self.hovered()
            }
        }

        fn disabled(&self) -> button::Style {
            let mut active = self.active();
            active.background = Color::from_rgb8(0xAE, 0xAE, 0xAE).into();
            active
            // button::Style {
            //     background: Color::from_rgb8(0x7D, 0x7D, 0x9D).into(),
            //     ..self.active()
            // }
        }
    }
}

#[allow(clippy::cast_precision_loss)]
mod dark {
    use iced::{Background, button, checkbox, Color, container, pick_list, progress_bar, scrollable, slider, text_input};
    use iced::slider::{Handle, HandleShape};
    use iced::text_input::Style;
    use iced_aw::tabs;

    use crate::SettingsBarStyle;
    use crate::utils::ColorExt;

    mod color {
        use iced::Color;

        pub const SURFACE: Color = color!(rgb 0x40 0x44 0x4B);

        pub const ACCENT: Color = color!(rgb 0x6F 0xFF 0xE9);

        pub const ACTIVE: Color = color!(rgb 0x62 0x79 0xCA);

        pub const HOVERED: Color = color!(rgb 0x77 0x87 0xD7);

        pub const BACKGROUND: Color = color!(rgb 0x36 0x39 0x3F);

        pub const BRIGHTER_THAN_BACKGROUND: Color = color!(rgb 0x3A 0x3C 0x43);

        pub const BRIGHTER_THAN_SURFACE: Color = color!(rgb 0x46 0x4A 0x51);

        pub mod tab_bar {
            use iced::Color;

            pub const BACKGROUND: Color = color!(rgb 0x2E 0x2F 0x37);
        }

        pub mod settings_bar {
            use iced::Color;

            pub const PROGRESS_BAR: Color = Color::from_rgb(
                0x3E as f32 / 255.0,
                0x3F as f32 / 255.0,
                0x47 as f32 / 255.0,
            );
        }

        pub mod alternating {
            use iced::Color;

            pub fn background(alternate: Option<bool>) -> Color {
                match alternate {
                    Some(true) => color!(rgb 0x30 0x33 0x35),
                    None | Some(false) => Color::TRANSPARENT,
                }
            }

            pub fn text(alternate: Option<bool>) -> Color {
                match alternate {
                    None => color!(rgb 0x00 0xFF 0x88),
                    Some(_) => Color::WHITE,
                }
            }

            pub fn hovered(alternate: Option<bool>) -> Color {
                match alternate {
                    None => color!(rgb 0xD1 0xD1 0x71),
                    Some(true) => color!(rgb 0x30 0x33 0x35),
                    Some(false) => color!(rgba 0 0 0 0),
                }
            }
        }
    }

    pub struct InitiativeTable(pub Option<bool>);

    impl container::StyleSheet for InitiativeTable {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 2.0,
                background: color::alternating::background(self.0).into(),
                border_color: Default::default(),
                text_color: color::alternating::text(self.0).into(),
                ..Container.style()
            }
        }
    }

    impl button::StyleSheet for InitiativeTable {
        fn active(&self) -> button::Style {
            button::Style {
                background: Color::TRANSPARENT.into(),
                text_color: color::alternating::text(self.0),
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            let mut style = self.active();
            match self.0 {
                None => {}
                Some(true) => {}
                Some(false) => {}
            };
            style
        }

        fn pressed(&self) -> button::Style {
            self.active()
        }
    }

    impl text_input::StyleSheet for InitiativeTable {
        fn active(&self) -> text_input::Style {
            text_input::Style {
                background: Color::TRANSPARENT.into(),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Default::default()
            }
        }

        fn focused(&self) -> text_input::Style {
            TextInput.focused()
        }

        fn placeholder_color(&self) -> Color {
            TextInput.placeholder_color()
        }

        fn value_color(&self) -> Color {
            TextInput.value_color()
        }

        fn selection_color(&self) -> Color {
            TextInput.selection_color()
        }

        fn hovered(&self) -> Style {
            TextInput.hovered()
        }
    }

    pub struct InitiativeTableBorder;

    impl container::StyleSheet for InitiativeTableBorder {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 5.0,
                border_width: 1.0,
                border_color: Color::BLACK.a(0.6),
                ..Container.style()
            }
        }
    }

    // todo rename this DefaultDark and combine all of em
    pub struct Container;

    impl container::StyleSheet for Container {
        fn style(&self) -> container::Style {
            container::Style {
                text_color: Some(Color::WHITE),
                background: Some(Background::Color(color::BACKGROUND)),
                ..Default::default()
            }
        }
    }

    pub struct TextInput;

    impl text_input::StyleSheet for TextInput {
        fn active(&self) -> text_input::Style {
            text_input::Style {
                background: Background::Color(color::SURFACE),
                border_radius: 2.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }
        }

        fn focused(&self) -> text_input::Style {
            text_input::Style {
                border_width: 1.0,
                border_color: color::ACCENT,
                ..self.active()
            }
        }

        fn placeholder_color(&self) -> Color {
            Color::from_rgb(0.4, 0.4, 0.4)
        }

        fn value_color(&self) -> Color {
            Color::WHITE
        }

        fn selection_color(&self) -> Color {
            color::ACTIVE
        }

        fn hovered(&self) -> text_input::Style {
            text_input::Style {
                border_width: 1.0,
                border_color: Color { a: 0.3, ..color::ACCENT },
                ..self.focused()
            }
        }
    }

    pub struct Scrollable;

    impl scrollable::StyleSheet for Scrollable {
        fn active(&self) -> scrollable::Scrollbar {
            scrollable::Scrollbar {
                background: Some(Background::Color(color::SURFACE)),
                border_radius: 2.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: color::ACTIVE,
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            }
        }

        fn hovered(&self) -> scrollable::Scrollbar {
            let active = self.active();
            scrollable::Scrollbar {
                background: Some(Background::Color(Color { a: 0.5, ..color::SURFACE })),
                scroller: scrollable::Scroller {
                    color: color::HOVERED,
                    ..active.scroller
                },
                ..active
            }
        }

        fn dragging(&self) -> scrollable::Scrollbar {
            let hovered = self.hovered();

            scrollable::Scrollbar {
                scroller: scrollable::Scroller {
                    color: Color::from_rgb(0.85, 0.85, 0.85),
                    ..hovered.scroller
                },
                ..hovered
            }
        }
    }

    pub struct Button;

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: color::ACTIVE.into(),
                border_radius: 4.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: color::HOVERED.into(),
                ..self.active()
            }
        }

        fn pressed(&self) -> button::Style {
            button::Style {
                border_width: 1.0,
                border_color: Color::WHITE,
                ..self.hovered()
            }
        }

        fn disabled(&self) -> button::Style {
            button::Style {
                background: Color::from_rgb8(0x52, 0x59, 0x9A).into(),
                ..self.active()
            }
        }
    }

    pub struct PickList;

    impl pick_list::StyleSheet for PickList {
        fn menu(&self) -> pick_list::Menu {
            pick_list::Menu {
                text_color: Color::WHITE,
                background: Background::Color(color::SURFACE),
                border_width: 1.0,
                border_color: [0.3, 0.3, 0.3].into(),
                selected_text_color: Color::WHITE,
                selected_background: Background::Color(color::ACTIVE),
            }
        }

        fn active(&self) -> pick_list::Style {
            pick_list::Style {
                text_color: Color::WHITE,
                background: Background::Color(color::SURFACE),
                border_radius: 3.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                icon_size: 0.0,
            }
        }

        fn hovered(&self) -> pick_list::Style {
            pick_list::Style {
                background: Background::Color(color::HOVERED),
                ..self.active()
            }
        }
    }

    pub struct Checkbox;

    impl checkbox::StyleSheet for Checkbox {
        fn active(&self, is_checked: bool) -> checkbox::Style {
            checkbox::Style {
                background: Background::Color(if is_checked {
                    color::ACTIVE
                } else {
                    color::SURFACE
                }),
                checkmark_color: Color::WHITE,
                border_radius: 2.0,
                border_width: 1.0,
                border_color: color::ACTIVE,
            }
        }

        fn hovered(&self, is_checked: bool) -> checkbox::Style {
            checkbox::Style {
                background: Background::Color(Color {
                    a: 0.8,
                    ..if is_checked { color::ACTIVE } else { color::SURFACE }
                }),
                ..self.active(is_checked)
            }
        }
    }

    pub struct Slider;

    impl Slider {}

    impl slider::StyleSheet for Slider {
        fn active(&self) -> slider::Style {
            slider::Style {
                rail_colors: (Color::WHITE, Color::TRANSPARENT),
                handle: Handle {
                    shape: HandleShape::Circle { radius: 7.0 },
                    color: color::SURFACE,
                    border_width: 1.0,
                    border_color: Color::WHITE,
                },
            }
        }

        fn hovered(&self) -> slider::Style {
            let mut style = self.active();
            style.handle.border_width = 1.5;
            style
        }

        fn dragging(&self) -> slider::Style {
            let mut style = self.hovered();
            style.handle.border_color = color::ACTIVE;
            style.handle.border_width += 0.5;
            style
        }
    }

    pub struct Tabs;

    impl tabs::StyleSheet for Tabs {
        fn active(&self, is_active: bool) -> tabs::Style {
            tabs::Style {
                background: None,
                border_color: None,
                border_width: 0.0,
                tab_label_background: Background::Color(
                    if is_active { color::BACKGROUND } else { color::SURFACE }
                ),
                tab_label_border_color: Default::default(),
                tab_label_border_width: 0.0,
                icon_color: Color::WHITE,
                text_color: Color::WHITE,
            }
        }

        fn hovered(&self, is_active: bool) -> tabs::Style {
            tabs::Style {
                background: None,
                border_color: None,
                border_width: 0.0,
                tab_label_background: Background::Color(
                    if is_active {
                        color::BRIGHTER_THAN_BACKGROUND
                    } else {
                        color::BRIGHTER_THAN_SURFACE
                    }
                ),
                tab_label_border_color: Default::default(),
                tab_label_border_width: 0.0,
                icon_color: Color::WHITE,
                text_color: Color::WHITE,
            }
        }
    }

    pub struct SettingsButton;

    impl button::StyleSheet for SettingsButton {
        fn active(&self) -> button::Style {
            button::Style {
                background: color::tab_bar::BACKGROUND.into(),
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }
    }

    pub struct SettingsContainer;

    impl container::StyleSheet for SettingsContainer {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(color::tab_bar::BACKGROUND)),
                ..Container.style()
            }
        }
    }

    impl progress_bar::StyleSheet for SettingsBarStyle {
        fn style(&self) -> progress_bar::Style {
            progress_bar::Style {
                background: color::settings_bar::PROGRESS_BAR.into(),
                bar: color::ACTIVE.into(),
                border_radius: 5.0,
            }
        }
    }

    pub struct TabButton;

    impl button::StyleSheet for TabButton {
        fn active(&self) -> button::Style {
            button::Style {
                background: color::BACKGROUND.into(),
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: Color::from_rgb8(
                    0x40,
                    0x40,
                    0x48,
                ).into(),
                ..self.active()
            }
        }

        fn disabled(&self) -> button::Style {
            button::Style {
                background: Color::from_rgb8(
                    0x46,
                    0x46,
                    0x57,
                ).into(),
                ..self.active()
            }
        }
    }

    // pub mod alt {
    //     use crate::utils::ColorExt;
    //
    //     use super::*;
    //
    //     pub struct Container<const N: usize>;
    //
    //     impl<const N: usize> container::StyleSheet for Container<N> {
    //         fn style(&self) -> container::Style {
    //             container::Style {
    //                 background: Some(Background::Color(color::alternating::background())),
    //                 ..super::Container.style()
    //             }
    //         }
    //     }
    //
    //     pub struct Button<const N: usize>(pub bool);
    //
    //     impl<const N: usize> button::StyleSheet for Button<N> {
    //         fn active(&self) -> button::Style {
    //             button::Style {
    //                 background: Color::TRANSPARENT.into(),
    //                 text_color: Color::WHITE,
    //                 // border_width: 0.7,
    //                 // border_color: Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0),
    //                 // border_radius: 1.0,
    //                 ..button::Style::default()
    //             }
    //         }
    //
    //         fn hovered(&self) -> button::Style {
    //             let mut style = self.active();
    //             if self.0 {
    //                 style.background = color::alternating::HOVERED[N].into();
    //             }
    //             style
    //         }
    //
    //         fn pressed(&self) -> button::Style {
    //             if self.0 {
    //                 button::Style {
    //                     border_width: 1.0,
    //                     border_radius: 3.0,
    //                     border_color: Color::WHITE.a(0.3),
    //                     ..self.active()
    //                 }
    //             } else {
    //                 self.active()
    //             }
    //         }
    //     }
    // }
}