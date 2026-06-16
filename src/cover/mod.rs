//! Turns a [`CoverConfig`] into the cover SVG. Pure and UI-agnostic.

pub mod config;
pub mod elements;
pub mod format;
pub mod layouts;
pub mod render;
pub mod typeset;

pub use config::CoverConfig;
pub use format::Format;
pub use layouts::Layout;
pub use render::render_cover_svg;
