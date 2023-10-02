use asr::{Address, Process, timer::TimerState, time::Duration};

// Loading = 0x55e230

struct State {
    is_timer_running: bool, // got it
    timer_vblanks: u32,
    level_id: u8,   // got it
    mode: u8,   // got it
    module: Module, // got it
    _gold_count: u32,
    medal_count: u32,
    goal_count: u8,
    is_loading: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Module {
    Unknown,    // really just one we don't care about, probably editor or maybe null if things go wrong
    Frontend,
    Game,
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

        // get currently loaded module
        let module = match process.read_pointer_path32::<u32>(base_addr, &vec!(0x139db4 as u32, 0x3c as u32)) {
            Ok(v) => {
                match process.read_pointer_path32::<u32>(base_addr, &vec!(0x139db4 as u32, v + 0x1c as u32)) {
                    Ok(v) => {
                        //asr::print_message(&format!("MODULE SIZE: {}", v));
                        if v == 0x43000 {
                            Module::Frontend
                        } else if v== 0x9c000 {
                            Module::Game
                        } else {
                            Module::Unknown
                        }
                    },
                    Err(_) => Module::Unknown, 
                }
            },
            Err(_) => Module::Unknown, 
        };

        match module {
            Module::Unknown => {
                Self {
                    is_timer_running: false,
                    timer_vblanks: 0,
                    level_id: 0,
                    mode: 0,
                    module,
                    _gold_count: 0,
                    medal_count: 0,
                    goal_count: 0,
                    is_loading: false,
                }
            },
            Module::Frontend => {
                let rider_id = match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, 0x190534 as u32)) {
                    Ok(v) => v,
                    Err(_) => 0xff,
                };

                if rider_id >= 0 && rider_id <= 9 {    // TODO: figure out secret skaters
                    let rider_profile = 0x850d0 + (rider_id as u32 * 0x38);
        
                    match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, rider_profile + 4 + 6)) {
                        Ok(v) => {
                            if v != 0 {
                                medal_count += 1
                            }
                        },
                        Err(_) => {},
                    };
                    match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, rider_profile + 4 + 7)) {
                        Ok(v) => {
                            if v != 0 {
                                medal_count += 1
                            }
                        },
                        Err(_) => {},
                    };
        
                    goal_count = match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, rider_profile)) {
                        Ok(v) => v,
                        Err(_) => 0,
                    };

                    gold_count = match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, rider_profile + 3)) {
                        Ok(v) => v,
                        Err(_) => 0,
                    };
                }

                Self {
                    is_timer_running: false,
                    timer_vblanks: 0,
                    level_id: 0,
                    mode: 0,
                    module,
                    _gold_count: gold_count as u32,
                    medal_count,
                    goal_count,
                    is_loading: false,
                }
            },
            Module::Game => {
                let rider_id = match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, 0x220a96 as u32)) {
                    Ok(v) => v,
                    Err(_) => 0xff,
                };

                if rider_id >= 0 && rider_id <= 9 {    // TODO: figure out secret skaters
                    let rider_profile = 0x221290 + (rider_id as u32 * 0x38);
        
                    match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, rider_profile + 4 + 6)) {
                        Ok(v) => {
                            if v != 0 {
                                medal_count += 1
                            }
                        },
                        Err(_) => {},
                    };
                    match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, rider_profile + 4 + 7)) {
                        Ok(v) => {
                            if v != 0 {
                                medal_count += 1
                            }
                        },
                        Err(_) => {},
                    };
        
                    goal_count = match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, rider_profile)) {
                        Ok(v) => v,
                        Err(_) => 0,
                    };

                    gold_count = match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, rider_profile + 3)) {
                        Ok(v) => v,
                        Err(_) => 0,
                    };
                }
        
                State {
                    is_timer_running: match process.read_pointer_path32::<bool>(base_addr, &vec!(0x139db4 as u32, 0xc6438 as u32)) {
                        Ok(v) => v,
                        Err(_) => false,
                    },
        
                    level_id: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, 0x220a93 as u32)) {
                        Ok(v) => v,
                        Err(_) => 0,
                    },
        
                    mode: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, 0x220ab0 as u32)) {
                        Ok(v) => v,
                        Err(_) => 0,
                    },
        
                    module,
        
                    // used for igt only, so clamp it to max run time
                    timer_vblanks: match process.read_pointer_path32::<u32>(base_addr, &vec!(0x139db4 as u32, 0xc61dc as u32)) {
                        Ok(v) => {
                            // possibly CBruce + 2cc0 is time left??
                            let level_id = match process.read::<u32>(base_addr + 0x15e8f0 as u32) {
                                Ok(v) => v.clamp(0, 13),
                                Err(_) => 0,
                            };
        
                            let is_comp = level_id == 6 || level_id == 7;
            
                            let max_time = if is_comp {
                                1 * 60 * 60 // 1 minutes * 60 seconds * 60 vblanks/sec
                            } else {
                                2 * 60 * 60 // 2 minutes * 60 seconds * 60 vblanks/sec
                            };
        
                            (v + 30).clamp(0, max_time)   // timer sticks at 2:00 for half a second, add 30 vblanks to account for this
                        },
                        Err(_) => 0,
                    },
        
                    _gold_count: gold_count as u32,
                    medal_count: medal_count,
                    goal_count: goal_count,

                    is_loading: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, 0xbc6b0 as u32)) {
                        Ok(v) => v != 0,
                        Err(_) => false,
                    },
                }
            }
        }
    }
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to MHPB!");
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

        if current_state.module != prev_state.module {
            asr::print_message(&format!("Module changed to {:?}!", current_state.module));
        }
        if current_state.timer_vblanks != prev_state.timer_vblanks {
            asr::print_message(&format!("Rider changed to {}!", current_state.timer_vblanks));
        }
        if current_state._gold_count != prev_state._gold_count {
            asr::print_message(&format!("Gold count changed to {}!", current_state._gold_count));
        }
        if current_state.medal_count != prev_state.medal_count {
            asr::print_message(&format!("Medal count changed to {}!", current_state.medal_count));
        }
        if current_state.goal_count != prev_state.goal_count {
            asr::print_message(&format!("Goal count changed to {}!", current_state.goal_count));
        }
        if current_state.level_id != prev_state.level_id {
            asr::print_message(&format!("Level changed to {}!", current_state.level_id));
        }
        if current_state.mode != prev_state.mode {
            asr::print_message(&format!("Mode changed to {}!", current_state.mode));
        }
        if current_state.is_timer_running != prev_state.is_timer_running {
            asr::print_message(&format!("Timer state changed to {}!", current_state.is_timer_running));
        }
        if current_state.is_loading != prev_state.is_loading {
            asr::print_message(&format!("Loading changed to {}!", current_state.is_loading));
        }

        // NOTE: when module changes, it takes a sec for stuff to make it over.  for this reason, reset on menu only when goals *changes to zero in frontend*

        /*match asr::timer::state() {
            TimerState::NotRunning => {
                if game_done {
                    game_done = false;
                }

                if current_state.mode == 1 && current_state.goal_count == 0 && current_state.level_id == 0 && current_state.is_timer_running {
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

                if current_state.screen == 6 && level_changed {
                    level_changed = false;
                    asr::timer::split();
                    asr::print_message(format!("Changed levels; splitting timer...").as_str());
                }

                // split when all medals collected 
                // TODO: add setting to only split when all goals and goals are collected
                if !game_done && current_state.medal_count == 3 {
                    game_done = true;
                    asr::timer::split();
                    asr::print_message(format!("Collected all medals; splitting timer...").as_str());
                }

                // reset when on a menu and no goals are complete on any skater
                if current_state.screen != 6 && State::check_for_reset(process, base_addr) {
                    asr::timer::reset();
                    asr::print_message(format!("Resetting timer...").as_str());

                    prev_igt = Duration::seconds(-1);
                    asr::timer::resume_game_time();
                }

                // calculate igt
                // commit run's time when either the timer has stopped (run ended) or current time is lower than previous while timer is running
                // FIXME: RECORD, THEN SUBTRACT TIMER FROM WHEN TIMER TRANSITIONS TO STARTED
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
        }*/

        prev_state = current_state;

        asr::future::next_tick().await;
    }
}