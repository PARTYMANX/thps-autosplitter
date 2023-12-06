use asr::{Address, Process, timer::TimerState};

struct State {
    level_id: u8,
    _goal_count: u8,
    chapter: u8,
    is_loading: bool,
    is_career_started: bool,
}

impl State {
    pub fn update(process: &Process, base_addr: Address) -> Self {
        State {
            level_id: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x36A788 as u32, 0x20 as u32, 0x5c4 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            is_career_started: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x36A788 as u32, 0x20 as u32, 0x592 as u32)) {
                Ok(v) => (v & 0x8) != 0,
                Err(_) => false,
            },

            _goal_count: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x36A788 as u32, 0x3a8 as u32, 0x24 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            chapter: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x36A788 as u32, 0x3a8 as u32, 0x3c as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            is_loading: match process.read_pointer_path32::<bool>(base_addr, &vec!(0x29851C as u32, 0x24 as u32, 0x174 as u32)) {
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

        if current_state.chapter != prev_state.chapter {
            asr::print_message(&format!("Chapter changed to {}!", current_state.chapter));
        }

        if current_state.level_id != prev_state.level_id {
            asr::print_message(&format!("level changed to {}!", current_state.level_id));
        }

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
                if current_state.is_career_started && !prev_state.is_career_started && current_state.level_id == 19 {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());
                }
            },
            TimerState::Paused | TimerState::Running => {
                if current_state.chapter != prev_state.chapter {
                    match current_state.chapter {
                        3 /* Manhattan */ |
                        6 /* Tampa */ | 
                        10 /* San Diego */ |
                        13 /* Hawaii */ |
                        16 /* Vancouver */ | 
                        17 /* Slam City Jam */ |
                        18 /* Vancouver 2 */ |
                        19 /* Moscow */ |
                        22 /* New Jersey 2 */ |
                        25 /* Pro Goals */ |
                        26 /* Eric's Line */ |
                        27 /* End */ => {
                        }
                        _ => {} // do nothing
                    }
                }

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