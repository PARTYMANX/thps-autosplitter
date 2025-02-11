use asr::settings::Gui;

#[derive(Gui)]
pub struct Settings {
    /// General Settings
    _general_settings: asr::settings::gui::Title,
    /// Auto Start
    ///
    /// Enable automatic starting
    pub auto_start: bool,
    /// Auto Split
    ///
    /// Enable automatic splitting
    pub auto_split: bool,
    /// Auto Reset
    ///
    /// Enable automatic resetting
    pub auto_reset: bool,
}