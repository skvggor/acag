//! Turns a [`CoverConfig`] into the cover SVG. Pure and UI-agnostic.

pub mod config;
pub mod elements;
pub mod layouts;
pub mod render;
pub mod typeset;

pub use config::CoverConfig;
pub use layouts::Layout;
pub use render::{CANVAS, render_cover_svg};
