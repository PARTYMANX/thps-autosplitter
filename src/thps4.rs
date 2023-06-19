use asr::{Address, Process, string::ArrayCString, timer::TimerState};

struct State {
    level_name: String,
    pro_points: u8,
    is_loading: bool,
}

impl State {
    pub fn update(process: &Process, base_addr: Address) -> Self {
        let mut level_name = "".to_string();
        let mut pro_points = 0;
        let mut is_loading = false;

        match process.read::<ArrayCString<16>>(base_addr + 0x6B6BF0 as u32) {
            Ok(v) => {
                match String::from_utf8(v.as_bytes().to_vec()) {
                    Ok(v) => level_name = v,
                    Err(err) => asr::print_message(format!("Error reading level name: {:?}", err).as_str()),
                }
            } 
            //Err(err) => asr::print_message(format!("Error reading level name: {:?}", err).as_str()),
            Err(_) => {}    // do nothing, we most likely lost the process
        }

        match process.read_pointer_path32::<u8>(base_addr, &vec!(0x6B5B48 as u32, 0x86c as u32, 0x20 as u32)) {
            Ok(v) => pro_points = v,
            //Err(err) => asr::print_message(format!("Error reading pro points: {:?}", err).as_str()),
            Err(_) => {}    // do nothing, we either lost the process or don't have a career initialized
        }

        match process.read::<bool>(base_addr + 0x6728C0 as u32) {
            Ok(v) => is_loading = v,
            //Err(err) => asr::print_message(format!("Error reading is loading: {:?}", err).as_str()),
            Err(_) => {}    // do nothing, we probably lost the process
        }

        State {
            level_name: level_name,
            pro_points: pro_points,
            is_loading: is_loading,
        }
    }
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THPS4!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();

    let mut prev_state = State::update(process, base_addr);

    loop {
        // update vars
        let current_state = State::update(process, base_addr);

        if current_state.level_name != prev_state.level_name {
            asr::print_message(format!("Level changed to {}!", current_state.level_name).as_str());
        }

        if current_state.pro_points != prev_state.pro_points {
            asr::print_message(format!("Pro points changed to {}!", current_state.pro_points).as_str());
        }

        if current_state.is_loading != prev_state.is_loading {
            asr::print_message(format!("Is loading changed to {}!", current_state.is_loading).as_str());
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
                if current_state.level_name == "sch" && prev_state.level_name == "skateshop" && current_state.pro_points == 0 {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());
                }
            },
            TimerState::Paused | TimerState::Running => {
                // split on level changes (except skateshop)
                if !current_state.level_name.is_empty() && current_state.level_name != prev_state.level_name && current_state.level_name != "skateshop" {
                    asr::timer::split();
                    asr::print_message(format!("Changed level; splitting timer...").as_str());
                }

                // split when pro challenge is completed (91 goals) TODO: adapt this for all goals/other categories
                if current_state.pro_points == 91 && prev_state.pro_points == 90 {
                    asr::timer::split();
                    asr::print_message(format!("Got 91 pro points; splitting timer...").as_str());
                }

                // reset when on skateshop with 0 pro points
                if current_state.level_name == "skateshop" && current_state.pro_points == 0 {
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