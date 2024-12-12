use super::PlayerTransition;
use crate::{integrations::lottie::asset::VelloLottieHandle, PlaybackOptions, Theme};

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: &'static str,
    pub asset: Option<VelloLottieHandle>,
    pub theme: Option<Theme>,
    pub options: Option<PlaybackOptions>,
    pub transitions: Vec<PlayerTransition>,
    /// Whether to reset the playhead when a transition exits this state
    pub reset_playhead_on_exit: bool,
    /// Whether to reset the playhead when a transition enters this state
    pub reset_playhead_on_start: bool,
}

impl PlayerState {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            asset: Default::default(),
            options: Default::default(),
            theme: Default::default(),
            transitions: vec![],
            reset_playhead_on_exit: false,
            reset_playhead_on_start: false,
        }
    }

    pub fn asset(mut self, asset: VelloLottieHandle) -> Self {
        self.asset.replace(asset);
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme.replace(theme);
        self
    }

    pub fn playback_options(mut self, options: PlaybackOptions) -> Self {
        self.options.replace(options);
        self
    }

    pub fn transition(mut self, transition: PlayerTransition) -> Self {
        self.transitions.push(transition);
        self
    }

    pub fn reset_playhead_on_exit(mut self) -> Self {
        self.reset_playhead_on_exit = true;
        self
    }

    pub fn reset_playhead_on_start(mut self) -> Self {
        self.reset_playhead_on_start = true;
        self
    }

    pub fn set_asset(mut self, asset: Option<VelloLottieHandle>) -> Self {
        self.asset = asset;
        self
    }

    pub fn set_theme(mut self, theme: Option<Theme>) -> Self {
        self.theme = theme;
        self
    }

    pub fn set_playback_options(mut self, options: Option<PlaybackOptions>) -> Self {
        self.options = options;
        self
    }

    pub fn set_transitions(mut self, transitions: Vec<PlayerTransition>) -> Self {
        self.transitions = transitions;
        self
    }

    pub fn set_reset_playhead_on_exit(mut self, reset: bool) -> Self {
        self.reset_playhead_on_exit = reset;
        self
    }

    pub fn set_reset_playhead_on_start(mut self, reset: bool) -> Self {
        self.reset_playhead_on_start = reset;
        self
    }

    pub fn get_asset(&self) -> Option<&VelloLottieHandle> {
        self.asset.as_ref()
    }

    pub fn get_theme(&self) -> Option<&Theme> {
        self.theme.as_ref()
    }

    pub fn get_playback_options(&self) -> Option<&PlaybackOptions> {
        self.options.as_ref()
    }

    pub fn get_transitions(&self) -> &Vec<PlayerTransition> {
        self.transitions.as_ref()
    }

    pub fn get_reset_playhead_on_exit(&self) -> bool {
        self.reset_playhead_on_exit
    }

    pub fn get_reset_playhead_on_start(&self) -> bool {
        self.reset_playhead_on_start
    }
}
