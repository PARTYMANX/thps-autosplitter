use asr::{Address, Process, string::ArrayCString, timer::TimerState};

struct State {
    last_cutscene: String,
    level_name: String,
    load_screen: String,
    total_classic_goals: u8,
    is_run_ended: u8,
    is_loading: bool,
}

impl State {
    pub fn update(process: &Process, base_addr: Address) -> Self {
        State {
            last_cutscene: match process.read::<ArrayCString<16>>(base_addr + 0x2F1058 as u32) {
                Ok(v) => {
                    match String::from_utf8(v.as_bytes().to_vec()) {
                        Ok(v) => v,
                        Err(err) => {
                            asr::print_message(format!("Error reading last cutscene name: {:?}", err).as_str());
                            "".to_string()
                        },
                    }
                },
                Err(_) => "".to_string(),
            },

            level_name: match process.read::<ArrayCString<16>>(base_addr + 0x3CE698 as u32) {
                Ok(v) => {
                    match String::from_utf8(v.as_bytes().to_vec()) {
                        Ok(v) => v,
                        Err(err) => {
                            asr::print_message(format!("Error reading level name: {:?}", err).as_str());
                            "".to_string()
                        },
                    }
                },
                Err(_) => "".to_string(),
            },

            load_screen: match process.read_pointer_path32::<ArrayCString<16>>(base_addr, &vec!(0x27E95C as u32, 0x0 as u32)) {
                Ok(v) => {
                    match String::from_utf8(v.as_bytes().to_vec()) {
                        Ok(v) => v,
                        Err(err) => {
                            asr::print_message(format!("Error reading load screen name: {:?}", err).as_str());
                            "".to_string()
                        },
                    }
                },
                Err(_) => "".to_string(),
            },

            total_classic_goals: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x2F3624 as u32, 0x20 as u32, 0x5EE as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            is_run_ended: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x2F3624 as u32, 0x1A18 as u32, 0xC as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            is_loading: match process.read_pointer_path32::<bool>(base_addr, &vec!(0x2FC49C as u32)) {
                Ok(v) => v,
                Err(_) => false,
            }
        }
    }
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THUG2!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();

    let mut prev_state = State::update(process, base_addr);

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
                // story
                if current_state.level_name == "BO" && prev_state.level_name == "TR" {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());
                }
                // classic
                if current_state.load_screen == "loadscrn_barcelona_classic" && prev_state.load_screen != current_state.load_screen && current_state.total_classic_goals == 0 {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());
                }
            },
            TimerState::Paused | TimerState::Running => {
                if current_state.level_name != prev_state.level_name && current_state.level_name != "mainmenu" {
                    asr::timer::split();
                    asr::print_message(format!("Changed level; splitting timer...").as_str());
                } 
                
                if current_state.last_cutscene == "SK_9" && current_state.last_cutscene != prev_state.last_cutscene {
                    asr::timer::split();
                    asr::print_message(format!("Final cutscene; splitting timer...").as_str());
                }

                if current_state.is_run_ended == 1 && current_state.is_run_ended != prev_state.is_run_ended && (current_state.total_classic_goals == 130 || current_state.total_classic_goals == 60 || current_state.total_classic_goals == 80) {
                    asr::timer::split();
                    asr::print_message(format!("End of classic mode; splitting timer...").as_str());
                }

                // reset when on main menu
                if current_state.level_name == "mainmenu" && prev_state.level_name != "mainmenu" {
                    asr::timer::reset();
                    asr::print_message(format!("Resetting timer...").as_str());
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