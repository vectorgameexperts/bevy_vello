mod asset_loader;

pub(crate) mod render;

pub mod asset;
pub use asset::{VelloLottie, VelloLottieHandle};

mod systems;
#[cfg(feature = "experimental-dotLottie")]
pub(crate) use systems::spawn_playheads;

mod parse;
pub use parse::{load_lottie_from_bytes, load_lottie_from_str};

mod lottie_ext;
pub use lottie_ext::LottieExt;

mod plugin;
pub(crate) use plugin::LottieIntegrationPlugin;

mod playback_options;
pub use playback_options::{
    PlaybackDirection, PlaybackLoopBehavior, PlaybackOptions, PlaybackPlayMode,
};

mod playhead;
pub use playhead::Playhead;

mod theme;
pub use theme::Theme;
