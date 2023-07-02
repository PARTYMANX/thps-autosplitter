use asr::{Address, Process, string::ArrayCString, timer::TimerState};

struct State {
    last_cutscene: String,
    level_name: String,
    chapter: u8,
    is_loading: bool,
}

impl State {
    pub fn update(process: &Process, base_addr: Address) -> Self {
        State {
            last_cutscene: match process.read::<ArrayCString<16>>(base_addr + 0x36A7C8 as u32) {
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

            level_name: match process.read::<ArrayCString<16>>(base_addr + 0x36B638 as u32) {
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

            chapter: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x36A788 as u32, 0x64c as u32)) {
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
                if current_state.last_cutscene == "Intro_02" {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());
                }
            },
            TimerState::Paused | TimerState::Running => {
                if current_state.level_name == "NJ" && prev_state.level_name == "cas_bedroom" {
                    // don't split when changing level to NJ from bedroom
                } else if current_state.level_name == "NY" && current_state.last_cutscene == "NY_03" {
                    // don't split when entering NY part two
                } else if current_state.level_name == "VC" && prev_state.level_name == "cas_bedroom" {
                    // don't split when at the end of Slam City Jam
                } else if current_state.level_name != prev_state.level_name && current_state.level_name != "nj_skateshop" && current_state.level_name != "skateshop" && current_state.chapter != 25 {
                    // split on level changes except to nj skateshop (cutscene level) or when pro goals are running
                    asr::timer::split();
                    asr::print_message(format!("Changed level; splitting timer...").as_str());
                } else if current_state.chapter == 25 && current_state.level_name != "nj_skateshop" && prev_state.level_name == "nj_skateshop" {
                    // split on pro goals start
                    asr::timer::split();
                    asr::print_message(format!("Started pro goals; splitting timer...").as_str());
                } else if current_state.chapter > 25 && current_state.level_name == "NJ" && current_state.chapter != prev_state.chapter {
                    // split on pro goals end
                    asr::timer::split();
                    asr::print_message(format!("Finished pro goals; splitting timer...").as_str());
                } else if current_state.last_cutscene == "NJ_10" && current_state.last_cutscene != prev_state.last_cutscene {
                    // split on game end
                    asr::timer::split();
                    asr::print_message(format!("Final cutscene; splitting timer...").as_str());
                }

                // reset when on skateshop with 0 pro points
                if current_state.level_name == "skateshop" && prev_state.level_name != "skateshop" {
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