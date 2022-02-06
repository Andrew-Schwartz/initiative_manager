use std::fmt::Display;
use std::str::FromStr;

use iced::{button, Button, Color, Column, Element, HorizontalAlignment, Length, Row, Rule, Scrollable, Space, Text, text_input, TextInput, Tooltip};
use iced_aw::Icon;
use iced_native::tooltip::Position;
use itertools::Itertools;
use rand::{Rng, thread_rng};
use regex::Regex;

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

#[derive(Default, Debug)]
pub struct TextInputState {
    pub state: text_input::State,
    pub content: String,
}

impl TextInputState {
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
        Self {
            state: Default::default(),
            value: true,
            states: [Icon::X, Icon::Check],
        }
    }
}

impl ToggleButtonState {
    pub fn new(is_enabled: bool, disabled: Icon, enabled: Icon) -> Self {
        Self {
            state: Default::default(),
            value: is_enabled,
            states: [disabled, enabled],
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

    pub fn button_with<M: Clone, F: FnOnce(Text) -> Text>(&mut self, text_config: F) -> Button<M> {
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

#[derive(Debug)]
pub enum Hp {
    Number(u32),
    Roll {
        n: u32,
        d: u32,
        plus: u32,
    },
    RollInProgress {
        n: u32,
        d: Option<u32>,
    },
}

impl Hp {
    pub fn into_number(self) -> u32 {
        match self {
            Self::Number(hp) => hp,
            Self::Roll { n, d, plus } => {
                let mut rng = thread_rng();
                (0..n).map(|_| rng.gen_range(1..=d)).sum::<u32>() + plus
            }
            Self::RollInProgress { .. } => unreachable!(),
        }
    }
}

impl FromStr for Hp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn next_non_empty<'a, I: Iterator<Item=&'a str>>(iter: &mut I) -> Option<&'a str> {
            match iter.next() {
                Some("") => next_non_empty(iter),
                Some(non_empty) => Some(non_empty),
                None => None,
            }
        }

        let result = s.parse().map(Self::Number)
            .map_err(|_| Err::<Self, _>(()))
            .or_else(|_| {
                // "20d" "20d10"  "20d10+100"  "20d10 + 100"  "20d+"
                let mut d_split = s.split('d');
                // "(20)d()"  "(20)d(10)"  "(20)d(10+100)"  "(20)d(10 + 100)"  "(20)d(+)"
                let n = d_split.next()
                    .ok_or(Err(()))?
                    .parse()
                    .map_err(|_| Err(()))?;
                println!("n = {:?}", n);
                // ""  "10"  "10+100"  "10 + 100"  "+"
                let mut plus_split = d_split.next()
                    .ok_or(Ok(Self::RollInProgress { n, d: None }))?
                    .split([' ', '+']);
                // "()"  "(10)"  "(10)+(100)"  "(10) ()+() (100)"  "()"
                let d = next_non_empty(&mut plus_split)
                    .ok_or(Ok(Self::RollInProgress { n, d: None }))?
                    .parse()
                    .map_err(|_| Err(()))?;
                // ""  "100" "()+() (100)"
                let plus = next_non_empty(&mut plus_split)
                    .ok_or(Ok(Self::Roll { n, d, plus: 0 }))?
                    .parse()
                    .map_err(|_| Err(()))?;

                match plus_split.next() {
                    None => Ok(Self::Roll { n, d, plus }),
                    Some(_) => Err(Err(())),
                }
            });
        match result {
            Ok(hp) => Ok(hp),
            Err(Ok(roll_in_progress)) => Ok(roll_in_progress),
            Err(Err(())) => Err(())
        }
    }
}