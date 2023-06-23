use std::thread::current;

use asr::{Address, Process, string::ArrayCString, timer::TimerState};

// looking for things:
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

struct State {
    level_name: String,
    comp_cash: i32,
    is_loading: bool,
    is_paused: bool,
    is_timer_paused: bool,
    is_timer_running: bool,
    level_id: u8,
    mode: u8,
    screen: u8,
    gold_count: u32,
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
        let skater_id = match process.read_pointer_path32::<i32>(base_addr, &vec!(0x1674b8 as u32, 0x2cc0 as u32)) {
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
            level_name: match process.read::<ArrayCString<16>>(base_addr + 0x29D198 as u32) {
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

            comp_cash: match process.read::<i32>(base_addr + 0x15CB8C as u32) {
                Ok(v) => v,
                Err(_) => 0,
            },

            is_loading: match process.read::<bool>(base_addr + 0x15E864 as u32) {
                Ok(v) => v,
                Err(_) => false,
            },

            is_paused: match process.read::<bool>(base_addr + 0x15E864 as u32) {
                Ok(v) => v,
                Err(_) => false,
            },

            is_timer_paused: match process.read::<bool>(base_addr + 0x29E050 as u32) {
                Ok(v) => v,
                Err(_) => false,
            },

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

            gold_count: gold_count,
            medal_count: medal_count,
            goal_count: goal_count,
        }
    }
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THPS2!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();

    let mut prev_state = State::update(process, base_addr);
    let mut game_done = false;
    let mut level_changed = false;

    loop {
        // update vars
        let current_state = State::update(process, base_addr);

        if current_state.gold_count != prev_state.gold_count {
            asr::print_message(format!("GOLD COUNT CHANGED TO {}", current_state.gold_count).as_str());
        }
        if current_state.medal_count != prev_state.medal_count {
            asr::print_message(format!("MEDAL COUNT CHANGED TO {}", current_state.medal_count).as_str());
        }
        if current_state.goal_count != prev_state.goal_count {
            asr::print_message(format!("GOAL COUNT CHANGED TO {}", current_state.goal_count).as_str());
        }

        if current_state.screen != prev_state.screen {
            asr::print_message(format!("SCREEN CHANGED TO {}", current_state.screen).as_str());
        }

        if current_state.mode != prev_state.mode {
            asr::print_message(format!("MODE CHANGED TO {}", current_state.mode).as_str());
        }

        if current_state.level_id != prev_state.level_id {
            asr::print_message(format!("LEVEL CHANGED TO {}", current_state.level_id).as_str());
        }

        if current_state.screen != 6 && State::check_for_reset(process, base_addr) {
            //asr::print_message(format!("WE SHOULD BE RESETTING").as_str());
        }

        if current_state.is_timer_paused != prev_state.is_timer_paused {
            //asr::print_message(format!("TIMER PAUSE STATE CHANGED TO {}", current_state.is_timer_paused).as_str());
        }

        if current_state.is_timer_running != prev_state.is_timer_running {
            asr::print_message(format!("TIMER STATE CHANGED TO {}", current_state.is_timer_running).as_str());
        }

        if current_state.is_paused != prev_state.is_paused {
            //asr::print_message(format!("PAUSE STATE CHANGED TO {}", current_state.is_paused).as_str());
        }

        match asr::timer::state() {
            TimerState::NotRunning => {
                if game_done {
                    game_done = false;
                }

                if current_state.mode == 1 && current_state.goal_count == 0 && current_state.level_id == 0 && current_state.is_timer_running {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());
                }
            },
            TimerState::Paused | TimerState::Running => {
                // split on level changes (except going to menu)
                if current_state.level_id != prev_state.level_id {
                    level_changed = true;
                }

                if current_state.screen == 6 && level_changed {
                    level_changed = false;
                    asr::timer::split();
                    asr::print_message(format!("Changed levels; splitting timer...").as_str());
                }

                // split when all medals collected
                if !game_done && current_state.medal_count == 3 {
                    game_done = true;
                    asr::timer::split();
                    asr::print_message(format!("Collected all goals and golds; splitting timer...").as_str());
                }

                // split when on a menu and no goals are complete on any skater
                if current_state.screen != 6 && State::check_for_reset(process, base_addr) {
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