mod asset_loader;

pub(crate) mod render;

mod asset;
pub use asset::{VelloSvg, VelloSvgHandle};

mod parse;
pub use parse::{load_svg_from_bytes, load_svg_from_str};

mod plugin;
pub(crate) use plugin::SvgIntegrationPlugin;
