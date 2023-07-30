use asr::{Address, Process, signature::Signature, string::ArrayCString, timer::TimerState};

// skate module, for me, is 0x4A3B5C
// level id: skate module -> 0x20 -> 0xb0
// flags are in skate module -> 0x20 -> down a ways
// cash: skate module -> 0x20 -> 0x21c
// goal manager: skate module -> 0x78
// HOW DO WE FIGURE THIS OUT?
// find where story goals are stored (maybe try finding flags)
// trace back what writes to flags
// 

struct Offsets {
    level_name: u32,
    loading_screen: u32,
    last_cutscene: u32,
    total_goals: u32,
    is_run_completed: u32,
}

impl Offsets {
    pub fn get(process: &Process, base_addr: Address, module_size: u64) -> Self {
        let level_name_ptr = Signature::<10>::new("8b 0c 24 51 68 ?? ?? ?? ?? e8").scan_process_range(process, (base_addr, module_size)).unwrap() + 5;
        let load_screen_ptr = Signature::<12>::new("a1 ?? ?? ?? ?? 73 05 b8 ?? ?? ?? ??").scan_process_range(process, (base_addr, module_size)).unwrap() + 1;
        let last_cutscene_ptr = Signature::<11>::new("74 15 ba ?? ?? ?? ?? 8b c6 2b d6").scan_process_range(process, (base_addr, module_size)).unwrap() + 3;
        let total_goals_ptr = Signature::<12>::new("39 5c 24 24 74 10 8b 0d ?? ?? ?? ??").scan_process_range(process, (base_addr, module_size)).unwrap() + 8;
        let is_run_completed_ptr = Signature::<11>::new("8b 04 85 ?? ?? ?? ?? 5e c2 04 00").scan_process_range(process, (base_addr, module_size)).unwrap() + 3;

        Offsets {
            level_name: match process.read::<u32>(level_name_ptr) {
                Ok(v) => v - 0x400000,
                Err(_) => 0,
            },
            loading_screen: match process.read::<u32>(load_screen_ptr) {
                Ok(v) => v - 0x400000,
                Err(_) => 0,
            },
            last_cutscene: match process.read::<u32>(last_cutscene_ptr) {
                Ok(v) => v - 0x400000,
                Err(_) => 0,
            },
            total_goals: match process.read::<u32>(total_goals_ptr) {
                Ok(v) => v - 0x400000,
                Err(_) => 0,
            },
            is_run_completed: match process.read::<u32>(is_run_completed_ptr) {
                Ok(v) => v - 0x400000,
                Err(_) => 0,
            },
        }
    }
}

struct State {
    level_name: String,
    loading_screen: String,
    last_cutscene: String,
    total_goals: u8,
    is_run_completed: u8,
}

impl State {
    pub fn update(process: &Process, base_addr: Address, offsets: &Offsets) -> Self {
        State {
            level_name: match process.read::<ArrayCString<16>>(base_addr + offsets.level_name as u32) {
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

            loading_screen: match process.read_pointer_path32::<ArrayCString<32>>(base_addr, &vec!(offsets.loading_screen as u32, 0x0 as u32)) {
                Ok(v) => {
                    match String::from_utf8(v.as_bytes().to_vec()) {
                        Ok(v) => v,
                        Err(err) => {
                            asr::print_message(format!("Error reading loading screen: {:?}", err).as_str());
                            "".to_string()
                        },
                    }
                },
                Err(_) => "".to_string(),
            },

            last_cutscene: match process.read::<ArrayCString<16>>(base_addr + offsets.last_cutscene as u32) {
                Ok(v) => {
                    match String::from_utf8(v.as_bytes().to_vec()) {
                        Ok(v) => v,
                        Err(err) => {
                            asr::print_message(format!("Error reading last cutscene: {:?}", err).as_str());
                            "".to_string()
                        },
                    }
                },
                Err(_) => "".to_string(),
            },

            total_goals: match process.read_pointer_path32::<u8>(base_addr, &vec!(offsets.total_goals as u32, 0x78 as u32, 0x38 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            is_run_completed: match process.read_pointer_path32::<u8>(base_addr, &vec!(offsets.is_run_completed + 0x30 as u32, 0x8 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
        }
    }
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THAW!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();
    let module_size = process.get_module_size(process_name).unwrap();

    let offsets = Offsets::get(process, base_addr, module_size);

    asr::print_message(format!("Got level name pointer: {}", offsets.level_name).as_str());
    asr::print_message(format!("Got skate pointer: {:X}", offsets.total_goals).as_str());

    let mut prev_state = State::update(process, base_addr, &offsets);

    loop {
        // update vars
        let current_state = State::update(process, base_addr, &offsets);

        // pause game time when loading, resume when done
        /*if current_state.is_loading && !prev_state.is_loading {
            asr::timer::pause_game_time();
            asr::print_message(format!("Starting Load...").as_str());
        } else if !current_state.is_loading && prev_state.is_loading {
            asr::timer::resume_game_time();
            asr::print_message(format!("Done Loading").as_str());
        }*/

        if current_state.level_name != prev_state.level_name {
            asr::print_message(format!("Level changed to {}!", current_state.level_name).as_str());
        }
        if current_state.last_cutscene != prev_state.last_cutscene {
            asr::print_message(format!("Last cutscene changed to {}!", current_state.last_cutscene).as_str());
        }
        if current_state.loading_screen != prev_state.loading_screen {
            asr::print_message(format!("Loading screen changed to {}!", current_state.loading_screen).as_str());
        }
        if current_state.total_goals != prev_state.total_goals {
            asr::print_message(format!("Total goals changed to {}!", current_state.total_goals).as_str());
        }
        if current_state.is_run_completed != prev_state.is_run_completed {
            asr::print_message(format!("Run completion state changed to {}!", current_state.is_run_completed).as_str());
        }

        match asr::timer::state() {
            TimerState::NotRunning => {
                /*if current_state.level_name == "sch" && prev_state.level_name == "skateshop" && current_state.pro_points == 0 {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());
                }*/
            },
            TimerState::Paused | TimerState::Running => {
                // split on level changes (except skateshop)
                /*if !current_state.level_name.is_empty() && current_state.level_name != prev_state.level_name && current_state.level_name != "skateshop" {
                    asr::timer::split();
                    asr::print_message(format!("Changed level; splitting timer...").as_str());
                }*/

                // split when pro challenge is completed (91 goals) TODO: adapt this for all goals/other categories
                /*if current_state.pro_points == 91 && prev_state.pro_points == 90 {
                    asr::timer::split();
                    asr::print_message(format!("Got 91 pro points; splitting timer...").as_str());
                }*/

                // reset when on skateshop with 0 pro points
                /*if current_state.level_name == "skateshop" && current_state.pro_points == 0 {
                    asr::timer::reset();
                    asr::print_message(format!("Resetting timer...").as_str());
                }*/
            },
            TimerState::Ended | TimerState::Unknown | _ => {
                // do nothing. maybe we should still run reset when it's ended?
            },
        }

        prev_state = current_state;

        asr::future::next_tick().await;
    }
}