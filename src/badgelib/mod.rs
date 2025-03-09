#![deny(unused_imports)]
#![forbid(absolute_paths_not_starting_with_crate)]

pub(crate) mod _icons;
pub(crate) mod _width;
pub(crate) mod badge;
pub(crate) mod color;
pub(crate) mod utils;

pub use badge::{Badge, DlPeriod};
pub use color::Color;
