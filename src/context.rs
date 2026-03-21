use asr::settings::Gui;

use crate::settings::Settings;

pub struct SplitterContext {
    settings: Settings,
}

impl SplitterContext {
    pub fn new() -> Self {
        let mut settings = Settings::register();

        settings.update();

        Self {
            settings,
        }
    }

    pub fn update(&mut self) {
        self.settings.update();
    }

    pub fn should_reset_on_close(&self) -> bool {
        self.settings.auto_reset_on_close
    }

    pub fn timer_start(&self) {
        if self.settings.auto_start {
            asr::timer::start();
            asr::print_message(&format!("Starting timer..."));
        }
    }

    pub fn timer_split(&self, reason: &str) {
        if self.settings.auto_split {
            asr::timer::split();
            asr::print_message(&format!("{}; splitting timer...", reason));
        }
    }

    pub fn timer_reset(&self, timer_stopped: bool) {
        if self.settings.auto_reset && (!timer_stopped || self.settings.auto_reset_after_stop) {
            asr::timer::reset();
            asr::print_message(&format!("Resetting timer..."));
        }
    }

    pub fn timer_pause_game_time(&self) {
        asr::timer::pause_game_time();
    }

    pub fn timer_resume_game_time(&self) {
        asr::timer::resume_game_time();
    }

    pub fn timer_set_game_time(&self, time: asr::time::Duration) {
        asr::timer::set_game_time(time);
    }
}