//! Core library for the article cover art generator.
//!
//! The GUI binary (`acag`) is a thin shell over this crate: everything that
//! turns a [`cover::CoverConfig`] into an SVG/PNG lives here and is unit-tested
//! independently of the slint UI.

pub mod cover;
pub mod design;
pub mod export;
pub mod preset;
pub mod raster;
