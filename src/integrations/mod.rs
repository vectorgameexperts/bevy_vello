//! Integrations for supported file types. These are included by cargo feature.
//!
//! # Features
//! - `svg` - Enables SVG loading and rendering
//! - `lottie` - Enable Lottie (JSON) loading and rendering
//! - `experimental-dotLottie` - Enables experimental support for dotLottie interactivity. WIP.

#[cfg(feature = "svg")]
pub mod svg;

#[cfg(feature = "lottie")]
pub mod lottie;

#[cfg(feature = "experimental-dotLottie")]
pub mod dot_lottie;

mod error;
pub use error::VectorLoaderError;

mod asset;
pub use asset::{VelloAsset, VelloAssetAnchor, VelloAssetHandle};

#[derive(Clone)]
pub enum VectorFile {
    #[cfg(feature = "svg")]
    Svg(std::sync::Arc<vello::Scene>),
    #[cfg(feature = "lottie")]
    Lottie(std::sync::Arc<velato::Composition>),
}
