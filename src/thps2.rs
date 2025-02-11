use asr::{Address, Process, timer::TimerState, time::Duration};

use crate::settings::Settings;

// looking for things TODO: implement these alternate offsets based off of the Activision Value release executable:
// level 0x561c90 / 0x161c90
// also possibly 0x56a898
// mode 0x561c74 / 0x161c74
// menu screen 0x561c78 / 0x161c78
// p1 skater 0x561c88 / 0x161c88
// possibly loading 0x56a880 / 0x16a880
// possibly current career 0x53c240
// also possibly current career 0x568a6c + (skater * 0x104)
// possibly skater 0x56c580
// maybe 0x56c580 + 100 is current skater, then 0x56c580 + 4 is career progress?
// orig line: 0x56c580 + 4 + *(0x56c580 + 100) + 0xc <- this appears to be the skater id
// 0x568a6c + (skater id * 0x104) gets you career?

// Loading = 0x55e230
// GLevel = 0x5674f8 (prevents splits)

struct State {
    is_timer_running: bool,
    timer_vblanks: u32,
    level_id: u8,
    mode: u8,
    screen: u8,
    _gold_count: u32,
    medal_count: u32,
    goal_count: u8,
}

impl State {
    pub fn check_for_reset(process: &Process, base_addr: Address) -> bool {
        for i in 0..15 {
            let goal_count = match process.read::<u8>(base_addr + 0x1656cc + (i * 0x104) as u32) {
                Ok(v) => v,
                Err(_) => 0,
            };

            if goal_count > 0 {
                return false;
            }
        }

        return true;
    }

    pub fn update(process: &Process, base_addr: Address) -> Self {
        let mut gold_count = 0;
        let mut medal_count = 0;
        let mut comp_all_cash = 0;
        let mut goal_count = 0;

        // get skater
        let skater_id = match process.read_pointer_path::<i32>(base_addr, asr::PointerSize::Bit32, &vec!(0x1674b8 as u64, 0x2cc0 as u64)) {
            Ok(v) => v,
            Err(_) => -1,
        };

        if skater_id > 0 && skater_id < 15 {    // TODO: figure out secret skaters
            let skater_profile = 0x1656cc + (skater_id as u32 * 0x104);
            let medal_offset = skater_profile + 0xc;

            for i in 0..13 {
                let is_comp = match process.read::<bool>(base_addr + 0x139040 + (i * 0x1ac) as u32) {
                    Ok(v) => v,
                    Err(_) => false,
                };

                if is_comp {
                    let medal_flags = match process.read::<u16>(base_addr + medal_offset + (i * 2) as u32) {
                        Ok(v) => v,
                        Err(_) => 0,
                    };

                    if (medal_flags & 0x1c00).count_ones() > 0 {
                        medal_count += 1;
                    }

                    if medal_flags & 0x400 > 1 {
                        gold_count += 1;
                    }

                    if medal_flags & 0x8000 > 1 {
                        comp_all_cash += 1;
                    }
                }
            }

            goal_count = match process.read::<u8>(base_addr + skater_profile as u32) {
                Ok(v) => v + comp_all_cash,
                Err(_) => 0,
            };
        }

        State {
            is_timer_running: match process.read::<bool>(base_addr + 0x16B238 as u32) {
                Ok(v) => v,
                Err(_) => false,
            },

            level_id: match process.read::<u8>(base_addr + 0x15e8f0 as u32) {
                Ok(v) => v,
                Err(_) => 0,
            },

            mode: match process.read::<u8>(base_addr + 0x15e8d4 as u32) {
                Ok(v) => v,
                Err(_) => 0,
            },

            screen: match process.read::<u8>(base_addr + 0x15e8d8 as u32) {
                Ok(v) => v,
                Err(_) => 0,
            },

            // used for igt only, so clamp it to max run time
            timer_vblanks: match process.read::<u32>(base_addr + 0x16af80 as u32) {
                Ok(v) => {
                    // possibly CBruce + 2cc0 is time left??
                    let level_id = match process.read::<u32>(base_addr + 0x15e8f0 as u32) {
                        Ok(v) => v.clamp(0, 13),
                        Err(_) => 0,
                    };

                    let is_comp = match process.read::<bool>(base_addr + 0x139040 + (level_id * 0x1ac) as u32) {
                        Ok(v) => v,
                        Err(_) => false,
                    };
    
                    let max_time = if is_comp {
                        1 * 60 * 60 // 1 minutes * 60 seconds * 60 vblanks/sec
                    } else {
                        2 * 60 * 60 // 2 minutes * 60 seconds * 60 vblanks/sec
                    };

                    (v + 30).clamp(0, max_time)   // timer sticks at 2:00 for half a second, add 30 vblanks to account for this
                },
                Err(_) => 0,
            },

            _gold_count: gold_count,
            medal_count: medal_count,
            goal_count: goal_count,
        }
    }
}

pub async fn run(process: &Process, process_name: &str, settings: &Settings) {
    asr::print_message("Attached to THPS2!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();

    let mut prev_state = State::update(process, base_addr);
    let mut game_done = false;
    let mut level_changed = false;

    let mut igt_accumulator: i64 = 0;   // igt in seconds
    let mut prev_igt = Duration::seconds(-1);

    loop {
        // update vars
        let current_state = State::update(process, base_addr);

        match asr::timer::state() {
            TimerState::NotRunning => {
                if game_done {
                    game_done = false;
                }

                if settings.auto_start && current_state.mode == 1 && current_state.goal_count == 0 && current_state.level_id == 0 && current_state.is_timer_running {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());

                    asr::timer::pause_game_time();
                    igt_accumulator = 0;
                }
            },
            TimerState::Paused | TimerState::Running => {
                // split on level changes (except going to menu)
                if current_state.level_id != prev_state.level_id {
                    level_changed = true;
                }

                if settings.auto_split && current_state.screen == 6 && level_changed {
                    level_changed = false;
                    asr::timer::split();
                    asr::print_message(format!("Changed levels; splitting timer...").as_str());
                }

                // split when all medals collected 
                // TODO: add setting to only split when all goals and goals are collected
                if settings.auto_split && !game_done && current_state.medal_count == 3 {
                    game_done = true;
                    asr::timer::split();
                    asr::print_message(format!("Collected all medals; splitting timer...").as_str());
                }

                // reset when on a menu and no goals are complete on any skater
                if settings.auto_reset && current_state.screen != 6 && State::check_for_reset(process, base_addr) {
                    asr::timer::reset();
                    asr::print_message(format!("Resetting timer...").as_str());

                    prev_igt = Duration::seconds(-1);
                    asr::timer::resume_game_time();
                }

                // calculate igt
                // commit run's time when either the timer has stopped (run ended) or current time is lower than previous while timer is running
                if (!current_state.is_timer_running && prev_state.is_timer_running) || (current_state.timer_vblanks < prev_state.timer_vblanks && prev_state.is_timer_running) {
                    igt_accumulator += prev_state.timer_vblanks as i64 / 60;
                }

                let igt_duration = if current_state.is_timer_running {
                    Duration::seconds(igt_accumulator + (current_state.timer_vblanks as i64 / 60))
                } else {
                    Duration::seconds(igt_accumulator)
                };

                // prevent excess messaging and only send igt when relevant
                if igt_duration != prev_igt {
                    prev_igt = igt_duration;
                    asr::timer::set_game_time(igt_duration);
                }
            },
            TimerState::Ended | TimerState::Unknown | _ => {
                // do nothing. maybe we should still run reset when it's ended?
            },
        }

        prev_state = current_state;

        asr::future::next_tick().await;
    }
}