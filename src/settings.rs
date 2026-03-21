use asr::settings::{gui::Title, Gui};

#[derive(Gui)]
pub struct Settings {
    /// General Settings
    _general_settings: Title,
    /// Auto Start
    ///
    /// Automatically start timer.
    #[default = true]
    pub auto_start: bool,
    /// Auto Split
    ///
    /// Automatically split timer.
    #[default = true]
    pub auto_split: bool,
    /// Auto Reset
    ///
    /// Automatically reset timer.
    #[default = true]
    pub auto_reset: bool,
    /// Allow Auto Reset After Stop
    ///
    /// Allows an automatic reset when the timer is stopped (e.g. When the run is complete).
    #[default = false]
    pub auto_reset_after_stop: bool,
    /// Reset Timer On Game Close
    ///
    /// Automatically resets the timer when the game closes
    #[default = true]
    pub auto_reset_on_close: bool,
}
