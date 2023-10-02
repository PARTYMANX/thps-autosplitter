use asr::{Address, Process, timer::TimerState, time::Duration};

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

        if matches!(module, Module::Frontend) {
            for i in 0..9 {
                let goal_count = match process.read_pointer_path32::<u8>(base_addr, &vec!(0x850d0 + (i * 0x38) as u32)) {
                    Ok(v) => v,
                    Err(_) => 0,
                };
    
                if goal_count > 0 {
                    return false;
                }
            }
    
            return true;
        }

        return false;
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

                if rider_id <= 9 {    // TODO: figure out secret skaters
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
                    is_loading: match process.read_pointer_path32::<u32>(base_addr, &vec!(0x139db4 as u32, 0x84824 as u32)) {
                        Ok(v) => v == 0,
                        Err(_) => false,
                    },
                }
            },
            Module::Game => {
                let rider_id = match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, 0x220a96 as u32)) {
                    Ok(v) => v,
                    Err(_) => 0xff,
                };

                if rider_id <= 9 {
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
                    } || match process.read_pointer_path32::<u8>(base_addr, &vec!(0x139db4 as u32, 0xbc598 as u32)) {
                        Ok(v) => v == 0,
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
    let mut start_vblank = 0;

    let mut prev_level = 0;

    loop {
        // update vars
        let mut current_state = State::update(process, base_addr);

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
                if game_done {
                    game_done = false;
                }

                if matches!(current_state.module, Module::Game) && current_state.mode == 0 && current_state.goal_count == 0 && current_state.level_id == 0 && current_state.is_timer_running {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());

                    //asr::timer::pause_game_time();
                    igt_accumulator = 0;
                    prev_level = 0;
                }
            },
            TimerState::Paused | TimerState::Running => {
                // level id will often change to 0 on menu, so preserve it from last gameplay
                if matches!(current_state.module, Module::Frontend) {
                    current_state.level_id = prev_level;
                }

                // keep level id stable if the module is still initializing
                if current_state.is_loading {
                    current_state.level_id = prev_state.level_id;
                }

                // make sure level is valid
                if matches!(current_state.module, Module::Game) && !current_state.is_loading {
                    // split on level changes (except going to menu)
                    if matches!(current_state.module, Module::Game) && current_state.level_id != prev_level {
                        level_changed = true;
                    }
                    prev_level = current_state.level_id;
                }

                if matches!(current_state.module, Module::Game) && level_changed {
                    level_changed = false;
                    asr::timer::split();
                    asr::print_message(format!("Changed levels; splitting timer...").as_str());
                }

                // split when all medals collected 
                // TODO: add setting to only split when all goals and goals are collected
                if matches!(current_state.module, Module::Game) && !game_done && current_state.medal_count == 2 {
                    game_done = true;
                    asr::timer::split();
                    asr::print_message(format!("Collected all medals; splitting timer...").as_str());
                }

                // reset when on a menu and no goals are complete on current rider
                if (matches!(current_state.module, Module::Frontend) || matches!(current_state.module, Module::Unknown)) && !current_state.is_loading && current_state.goal_count == 0 {
                    asr::timer::reset();
                    asr::print_message(format!("Resetting timer...").as_str());

                    prev_igt = Duration::seconds(-1);
                    //asr::timer::resume_game_time();
                }

                // calculate igt
                // commit run's time when either the timer has stopped (run ended) or current time is lower than previous while timer is running
                /*if matches!(current_state.module, Module::Game) {
                    if current_state.is_timer_running && !prev_state.is_timer_running {
                        start_vblank = current_state.timer_vblanks;
                    }

                    if (!current_state.is_timer_running && prev_state.is_timer_running) || (current_state.timer_vblanks < prev_state.timer_vblanks && prev_state.is_timer_running) {
                        igt_accumulator += (prev_state.timer_vblanks - start_vblank) as i64 / 60;
                        start_vblank = 0;
                    }
    
                    let igt_duration = if current_state.is_timer_running {
                        Duration::seconds(igt_accumulator + ((current_state.timer_vblanks - start_vblank) as i64 / 60))
                    } else {
                        Duration::seconds(igt_accumulator)
                    };
    
                    // prevent excess messaging and only send igt when relevant
                    if igt_duration != prev_igt {
                        prev_igt = igt_duration;
                        asr::timer::set_game_time(igt_duration);
                    }
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