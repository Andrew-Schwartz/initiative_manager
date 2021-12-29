use std::array::IntoIter;
use std::fmt::Display;

use iced::{Color, Column, Length, Row, Scrollable, Space};

pub trait SpacingExt {
    fn push_space<L: Into<Length>>(self, length: L) -> Self;
}

impl<'a, Message: 'a> SpacingExt for Column<'a, Message> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(Space::with_height(length.into()))
    }
}

impl<'a, Message: 'a> SpacingExt for Row<'a, Message> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(Space::with_width(length.into()))
    }
}

impl<'a, Message: 'a> SpacingExt for Scrollable<'a, Message> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(Space::with_height(length.into()))
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

pub trait Tap: Sized {
    fn tap<F: FnOnce(Self) -> Self>(self, f: F) -> Self {
        f(self)
    }
}

impl<T: Sized> Tap for T {}

pub trait IterExt: Iterator + Sized {
    fn none<P: FnMut(Self::Item) -> bool>(mut self, predicate: P) -> bool {
        !self.any(predicate)
    }
}

impl<I: Iterator + Sized> IterExt for I {}