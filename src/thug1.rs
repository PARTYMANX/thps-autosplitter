use std::u64;

use asr::{Address, Process, timer::TimerState, string::ArrayCString};

struct State {
    has_played_intro: bool,
    level_id: u8,
    _goal_count: u8,
    chapter: u8,
    is_loading: bool,
    is_career_started: bool,
}

impl State {
    pub fn update(process: &Process, base_addr: Address) -> Self {
        State {
            has_played_intro: match process.read::<ArrayCString<16>>(base_addr + 0x36A7C8 as u32) {
                Ok(v) => {
                    match String::from_utf8(v.as_bytes().to_vec()) {
                        Ok(v) => v == "Intro_02",
                        Err(err) => {
                            asr::print_message(format!("Error reading last cutscene name: {:?}", err).as_str());
                            false
                        },
                    }
                },
                Err(_) => false,
            },

            level_id: match process.read_pointer_path::<u8>(base_addr, asr::PointerSize::Bit32, &vec!(0x36A788 as u64, 0x20 as u64, 0x5c4 as u64)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            is_career_started: match process.read_pointer_path::<u8>(base_addr, asr::PointerSize::Bit32, &vec!(0x36A788 as u64, 0x20 as u64, 0x592 as u64)) {
                Ok(v) => (v & 0x8) != 0,
                Err(_) => false,
            },

            _goal_count: match process.read_pointer_path::<u8>(base_addr, asr::PointerSize::Bit32, &vec!(0x36A788 as u64, 0x3a8 as u64, 0x24 as u64)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            chapter: match process.read_pointer_path::<u8>(base_addr, asr::PointerSize::Bit32, &vec!(0x36A788 as u64, 0x3a8 as u64, 0x3c as u64)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            is_loading: match process.read_pointer_path::<bool>(base_addr, asr::PointerSize::Bit32, &vec!(0x29851C as u64, 0x24 as u64, 0x174 as u64)) {
                Ok(v) => v,
                Err(_) => false,
            }
        }
    }
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THUG1!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();

    let mut prev_state = State::update(process, base_addr);

    // these are used to prevent double splitting while preserving timing
    let mut story_flags = [
        false, // manhattan
        false, // tampa
        false, // san diego
        false, // hawaii
        false, // vancouver
        false, // slam city jam
        false, // vancouver 2
        false, // moscow
        false, // new jersey 2
        false, // pro goals
        false, // eric's line
        false, // eric's line done
    ];

    loop {
        // update vars
        let current_state = State::update(process, base_addr);

        // pause game time when loading, resume when done
        if current_state.is_loading && !prev_state.is_loading {
            asr::timer::pause_game_time();
            asr::print_message(format!("Starting Load...").as_str());
        } else if !current_state.is_loading && prev_state.is_loading {
            asr::timer::resume_game_time();
            asr::print_message(format!("Done Loading").as_str());
        }

        match asr::timer::state() {
            TimerState::NotRunning => {
                story_flags.fill(false);
                if current_state.has_played_intro && !prev_state.has_played_intro {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());
                }
            },
            TimerState::Paused | TimerState::Running => {
                if !story_flags[0] && current_state.chapter == 3 && current_state.level_id == 2 {
                    asr::timer::split();
                    asr::print_message(format!("Started Manhattan; splitting timer...").as_str());
                    story_flags[0] = true;
                }

                if !story_flags[1] && current_state.chapter == 6 && current_state.level_id == 3 {
                    asr::timer::split();
                    asr::print_message(format!("Started Tampa; splitting timer...").as_str());
                    story_flags[1] = true;
                }

                if !story_flags[2] && current_state.chapter == 10 && current_state.level_id == 4 {
                    asr::timer::split();
                    asr::print_message(format!("Started San Diego; splitting timer...").as_str());
                    story_flags[2] = true;
                }

                if !story_flags[3] && current_state.chapter == 13 && current_state.level_id == 5 {
                    asr::timer::split();
                    asr::print_message(format!("Started Hawaii; splitting timer...").as_str());
                    story_flags[3] = true;
                }

                if !story_flags[4] && current_state.chapter == 16 && current_state.level_id == 6 {
                    asr::timer::split();
                    asr::print_message(format!("Started Vancouver; splitting timer...").as_str());
                    story_flags[4] = true;
                }

                if !story_flags[5] && current_state.chapter == 17 && current_state.level_id == 7 {
                    asr::timer::split();
                    asr::print_message(format!("Started Slam City Jam; splitting timer...").as_str());
                    story_flags[5] = true;
                }

                if !story_flags[6] && current_state.chapter == 18 && current_state.level_id == 6 {
                    asr::timer::split();
                    asr::print_message(format!("Started Vancouver 2; splitting timer...").as_str());
                    story_flags[6] = true;
                }

                if !story_flags[7] && current_state.chapter == 19 && current_state.level_id == 8 {
                    asr::timer::split();
                    asr::print_message(format!("Started Moscow; splitting timer...").as_str());
                    story_flags[7] = true;
                }
                
                if !story_flags[8] && current_state.chapter == 22 && current_state.level_id == 1 {
                    asr::timer::split();
                    asr::print_message(format!("Started New Jersey 2; splitting timer...").as_str());
                    story_flags[8] = true;
                }

                if !story_flags[9] && current_state.chapter == 25 && current_state.level_id != 20 {
                    asr::timer::split();
                    asr::print_message(format!("Started Pro Goals; splitting timer...").as_str());
                    story_flags[9] = true;
                }

                if !story_flags[10] && current_state.chapter == 26 && current_state.level_id == 1 {
                    asr::timer::split();
                    asr::print_message(format!("Started Eric's Line; splitting timer...").as_str());
                    story_flags[10] = true;
                }

                if !story_flags[11] && current_state.chapter == 27 {
                    asr::timer::split();
                    asr::print_message(format!("Finished Story; splitting timer...").as_str());
                    story_flags[11] = true;
                }

                // reset when on main menu with a career not started
                if current_state.level_id == 0 && !current_state.is_career_started {
                    asr::timer::reset();
                    asr::print_message(format!("Resetting timer...").as_str());
                    story_flags.fill(false);
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