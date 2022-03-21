use std::fmt::Display;
use std::str::FromStr;

use iced::{button, Button, Checkbox, Color, Column, Element, HorizontalAlignment, Length, Row, Rule, Scrollable, Space, Text, text_input, TextInput, Tooltip};
use iced_aw::Icon;
use iced_native::tooltip::Position;
use itertools::Itertools;
use once_cell::sync::Lazy;
use rand::{Rng, thread_rng};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::Message;

pub trait SpacingExt {
    fn push_space<L: Into<Length>>(self, length: L) -> Self;

    fn push_rule(self, spacing: u16) -> Self;
}

impl<'a, Message: 'a> SpacingExt for Column<'a, Message> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(Space::with_height(length.into()))
    }

    fn push_rule(self, spacing: u16) -> Self {
        self.push(Rule::horizontal(spacing))
    }
}

impl<'a, Message: 'a> SpacingExt for Row<'a, Message> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(Space::with_width(length.into()))
    }

    fn push_rule(self, spacing: u16) -> Self {
        self.push(Rule::vertical(spacing))
    }
}

impl<'a, Message: 'a> SpacingExt for Scrollable<'a, Message> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(Space::with_height(length.into()))
    }

    fn push_rule(self, spacing: u16) -> Self {
        self.push(Rule::horizontal(spacing))
    }
}

pub trait ColorExt {
    fn r(self, r: f32) -> Self;
    fn g(self, g: f32) -> Self;
    fn b(self, b: f32) -> Self;
    fn a(self, a: f32) -> Self;
}

impl ColorExt for Color {
    fn r(mut self, r: f32) -> Self {
        self.r = r;
        self
    }

    fn g(mut self, g: f32) -> Self {
        self.g = g;
        self
    }

    fn b(mut self, b: f32) -> Self {
        self.b = b;
        self
    }

    fn a(mut self, a: f32) -> Self {
        self.a = a;
        self
    }
}

pub trait TryRemoveExt<T> {
    fn try_remove(&mut self, index: usize) -> Option<T>;
}

impl<T> TryRemoveExt<T> for Vec<T> {
    fn try_remove(&mut self, index: usize) -> Option<T> {
        if self.len() > index {
            Some(self.remove(index))
        } else {
            None
        }
    }
}

pub trait ListGrammaticallyExt: ExactSizeIterator + Sized {
    fn list_grammatically(self) -> String where Self::Item: Display {
        if self.len() == 0 { return String::new(); }
        let last = self.len() - 1;
        self.enumerate()
            .fold(String::new(), |mut acc, (i, new)| {
                if i != 0 {
                    acc.push_str(if i == last {
                        if i == 1 {
                            " and "
                        } else {
                            ", and "
                        }
                    } else {
                        ", "
                    });
                }
                acc = format!("{}{}", acc, new);
                acc
            })
    }
}

impl<T: Display, I: ExactSizeIterator<Item=T>> ListGrammaticallyExt for I {}

pub trait Tap {
    fn tap<T, F: FnOnce(Self) -> T>(self, f: F) -> T where Self: Sized {
        f(self)
    }

    fn tap_if<F: FnOnce(Self) -> Self>(self, condition: bool, f: F) -> Self where Self: Sized {
        if condition {
            f(self)
        } else {
            self
        }
    }

    fn tap_if_some<T, F: FnOnce(Self, T) -> Self>(self, option: Option<T>, f: F) -> Self where Self: Sized {
        if let Some(t) = option {
            f(self, t)
        } else {
            self
        }
    }

    fn tap_ref<T, F: FnOnce(&Self) -> T>(&self, f: F) -> T {
        f(self)
    }
}

impl<T> Tap for T {}

pub trait IterExt: Iterator + Sized {
    fn none<P: FnMut(Self::Item) -> bool>(mut self, predicate: P) -> bool {
        !self.any(predicate)
    }
}

impl<I: Iterator + Sized> IterExt for I {}

#[derive(Default, Debug, Copy, Clone, Deserialize, Serialize)]
pub struct Hidden<T>(pub T, pub bool);

impl<T> From<T> for Hidden<T> {
    fn from(t: T) -> Self {
        Hidden(t, false)
    }
}

pub trait MakeHidden: Sized {
    fn hidden(self, hidden: bool) -> Hidden<Self> {
        Hidden(self, hidden)
    }
}

impl<T: Sized> MakeHidden for T {}

pub fn checkbox<F: 'static + Fn(bool) -> Message>(is_checked: bool, f: F) -> Checkbox<Message> {
    Checkbox::new(is_checked, String::new(), f).spacing(0)
}

#[derive(Default, Debug)]
pub struct TextInputState {
    pub state: text_input::State,
    pub content: String,
}

impl TextInputState {
    pub fn focused() -> Self {
        Self {
            state: text_input::State::focused(),
            content: String::default(),
        }
    }

    pub fn text_input<M, F>(&mut self, placeholder: &str, on_change: F) -> TextInput<M>
        where M: Clone,
              F: 'static + Fn(String) -> M
    {
        TextInput::new(
            &mut self.state,
            placeholder,
            self.content.as_str(),
            on_change,
        )
    }
}

#[derive(Debug)]
pub struct ToggleButtonState {
    pub state: button::State,
    pub value: bool,
    pub states: [Icon; 2],
}

impl Default for ToggleButtonState {
    fn default() -> Self {
        Self::new(false)
    }
}

impl ToggleButtonState {
    pub const DEFAULT_STATES: [Icon; 2] = [Icon::X, Icon::Check];

    pub fn new(is_enabled: bool) -> Self {
        Self::new_with(is_enabled, Self::DEFAULT_STATES)
    }

    pub fn new_with(is_enabled: bool, disabled_enabled: [Icon; 2]) -> Self {
        Self {
            state: Default::default(),
            value: is_enabled,
            states: disabled_enabled,
        }
    }

    pub fn button<M: Clone>(&mut self) -> Button<M> {
        let label = self.states[usize::from(self.value)];
        Button::new(
            &mut self.state,
            Text::new(label)
                .font(iced_aw::ICON_FONT)
                .horizontal_alignment(HorizontalAlignment::Center),
        )
    }

    pub fn button_with<'a, M, E, F>(&'a mut self, text_config: F) -> Button<'a, M>
        where
            M: Clone,
            E: Into<Element<'a, M>>,
            F: FnOnce(Text) -> E
    {
        let label = self.states[usize::from(self.value)];
        Button::new(
            &mut self.state,
            text_config(Text::new(label)
                .font(iced_aw::ICON_FONT)
                .horizontal_alignment(HorizontalAlignment::Center)),
        )
    }

    pub fn invert(&mut self) {
        self.value = !self.value;
    }
}

pub trait TooltipExt<'a, Message>: Into<Element<'a, Message>> {
    fn tooltip<S: ToString>(self, tooltip: S, position: Position) -> Tooltip<'a, Message> {
        Tooltip::new(self, tooltip, position)
    }
}

impl<'a, Message, E: Into<Element<'a, Message>>> TooltipExt<'a, Message> for E {}

pub fn censor_name(name: &str) -> String {
    const CENSOR: [char; 26] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
        'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];
    let mut rng = thread_rng();
    Regex::new(r#"\s+"#).unwrap()
        .split(name)
        .map(|word| (0..word.len() + 1 - rng.gen_range(0..2))
            .map(|_| CENSOR[rng.gen_range(0..26)])
            .collect::<String>())
        .join(" ")
}

#[derive(Debug, Copy, Clone)]
pub enum HpPart {
    Number(u32),
    // NumberInProgress,
    Roll {
        n: u32,
        d: u32,
    },
    RollInProgress {
        n: u32,
    },
}

impl HpPart {
    pub fn into_number<R: Rng>(self, rng: &mut R) -> Option<u32> {
        match self {
            Self::Number(hp) => Some(hp),
            Self::Roll { n, d } => Some((0..n).map(|_| rng.gen_range(1..=d)).sum()),
            Self::RollInProgress { .. } => None,
        }
    }
}

impl FromStr for HpPart {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() { return Ok(Self::Number(0)); }
        let mut d_split = s.split("d");
        let n = d_split.next()
            .ok_or(())?
            .parse()
            .map_err(|_| ())?;
        let d = d_split.next();
        if d_split.count() != 0 {
            return Err(());
        }
        match d {
            None => Ok(Self::Number(n)),
            Some("") => Ok(Self::RollInProgress { n }),
            Some(d) => {
                let d = d.parse()
                    .map_err(|_| ())?;
                Ok(Self::Roll { n, d })
            }
        }
    }
}

#[derive(Debug)]
pub struct Hp(Vec<HpPart>);

impl Hp {
    pub fn new(hp: u32) -> Self {
        Self(vec![HpPart::Number(hp)])
    }

    pub fn into_number(self) -> Option<u32> {
        let mut rng = rand::thread_rng();
        self.0.into_iter()
            .map(|hp| hp.into_number(&mut rng))
            .fold_options(0, |a, b| a + b)
    }
}

impl FromStr for Hp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static PLUS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\s*\+\s*"#).unwrap());
        let vec = PLUS_REGEX.split(s)
            .map(HpPart::from_str)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self(vec))
    }
}
