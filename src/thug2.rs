use asr::{Address, Process, timer::TimerState};

struct State {
    level_id: u8,
    total_classic_goals: u8,
    classic_triangle_goals: u8,
    is_run_ended: bool,
    is_game_finished: bool,
    is_loading: bool,
    is_story_started: bool,
    story_points: u16,
    _story_difficulty: Difficulty,
    classic_difficulty: Difficulty,
}

struct ScriptStruct {
    _unk: u32,
    phead: u32,
}

impl ScriptStruct {
    pub fn read(process: &Process, addr: u32) -> Self {
        Self {
            _unk: match process.read_pointer_path32::<u32>(addr, &vec!(0x0 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            phead: match process.read_pointer_path32::<u32>(addr, &vec!(0x4 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
        }
    }
}

struct ScriptComponent {
    _unk: u8,
    _ttype: u8,
    _size: u16,
    name: u32,
    data: u32,
    pnext: u32,
}

impl ScriptComponent {
    pub fn read(process: &Process, addr: u32) -> Self {
        Self {
            _unk: match process.read_pointer_path32::<u8>(addr, &vec!(0x0 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            _ttype: match process.read_pointer_path32::<u8>(addr, &vec!(0x1 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            _size: match process.read_pointer_path32::<u16>(addr, &vec!(0x2 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            name: match process.read_pointer_path32::<u32>(addr, &vec!(0x4 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            data: match process.read_pointer_path32::<u32>(addr, &vec!(0x8 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            pnext: match process.read_pointer_path32::<u32>(addr, &vec!(0xc as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)] 
enum Difficulty {
    UNKNOWN,
    EASY,
    NORMAL,
    SICK,
}

fn read_difficulty_levels(process: &Process, addr: u32) -> (Difficulty, Difficulty) {
    let stru = ScriptStruct::read(process, addr);
    let mut comp = stru.phead;
    let mut story_difficulty = Difficulty::UNKNOWN;
    let mut classic_difficulty = Difficulty::UNKNOWN;
    while comp != 0 {
        let component = ScriptComponent::read(process, comp);

        if component.name == 0x4da4937b {   // "career"
            if component.data == 0 {
                story_difficulty = Difficulty::EASY;
            } else if component.data == 1 {
                story_difficulty = Difficulty::NORMAL;
            } else if component.data == 2 {
                story_difficulty = Difficulty::SICK;
            }
        } else if component.name == 0x4C10DE52 {    // "classic"
            if component.data == 0 {
                classic_difficulty = Difficulty::NORMAL;
            } else if component.data == 1 {
                classic_difficulty = Difficulty::SICK;
            }
        }

        comp = component.pnext;
    }

    (story_difficulty, classic_difficulty)
}

fn read_goal_params(process: &Process, addr: u32) -> (Difficulty, Difficulty) {
    let stru = ScriptStruct::read(process, addr);
    let mut comp = stru.phead;
    let mut result = (Difficulty::UNKNOWN, Difficulty::UNKNOWN);
    while comp != 0 {
        let component = ScriptComponent::read(process, comp);

        if component.name == 0xb13d98d3 {   // "difficulty_levels"
            result = read_difficulty_levels(process, component.data);
        }

        comp = component.pnext;
    }

    result
}

fn get_difficulties(process: &Process, addr: u32) -> (Difficulty, Difficulty) {
    let stru = ScriptStruct::read(process, addr);
    let mut result = (Difficulty::UNKNOWN, Difficulty::UNKNOWN);
    let mut comp = stru.phead;
    while comp != 0 {
        let component = ScriptComponent::read(process, comp);

        if component.name == 0x23d4170a {   // "goalmanager_params"
            result = read_goal_params(process, component.data);
        } /*else if component.ttype >> 1 == 0xa {
            if let Some(goal) = read_goal(process, component.data) {
                if goal.complete && goal.unk2 == 1 && state.completed_goals.borrow_mut().insert(component.name) {
                    if matches!(goal.mode, Gamemode::CAREER) {
                        state.story_goals += 1;
                    } else {
                        state.classic_goals += 1;
                    }
                }

                //if goal.is_locked {
                //    state.locked_goals.borrow_mut().insert(component.name);
                //}
            }
        }*/

        comp = component.pnext;
    }

    result
}

impl State {
    pub fn update(process: &Process, base_addr: Address) -> Self {
        let (_story_difficulty, classic_difficulty) = if let Ok(addr) = process.read_pointer_path32::<u32>(base_addr, &vec!(0x3ce478 as u32, 0x38c as u32, 0x14 as u32)) {
            get_difficulties(process, addr)
        } else {
            (Difficulty::UNKNOWN, Difficulty::UNKNOWN)
        };

        State {
            level_id: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x3ce478 as u32, 0x20 as u32, 0x630 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            is_game_finished: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x3ce478 as u32, 0x20 as u32, 0x610 as u32)) {
                Ok(v) => v & 0x40 != 0,
                Err(_) => false,
            },

            total_classic_goals: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x3ce478 as u32, 0x20 as u32, 0x5EE as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            classic_triangle_goals: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x3ce478 as u32, 0x20 as u32, 0x5E4 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            is_run_ended: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x2F3624 as u32, 0x1A18 as u32, 0xC as u32)) {
                Ok(v) => v != 0,
                Err(_) => false,
            },

            is_loading: match process.read_pointer_path32::<bool>(base_addr, &vec!(0x2FC49C as u32)) {
                Ok(v) => v,
                Err(_) => false,
            },

            is_story_started: match process.read_pointer_path32::<u8>(base_addr, &vec!(0x3ce478 as u32, 0x20 as u32, 0x634 as u32)) {
                Ok(v) => v & 0x1 != 0,
                Err(_) => false,
            },

            story_points: match process.read_pointer_path32::<u16>(base_addr, &vec!(0x3ce478 as u32, 0x20 as u32, 0x5d8 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            _story_difficulty,
            classic_difficulty,
        }
    }
}

enum Gamemode {
    NONE,
    CAREER,
    CLASSIC,
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THUG2!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();

    let mut prev_state = State::update(process, base_addr);

    let mut mode = Gamemode::NONE;

    loop {
        // update vars
        let current_state = State::update(process, base_addr);

        if current_state.story_points != prev_state.story_points {
            asr::print_message(format!("Story points changed to {}", current_state.story_points).as_str());
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
                // story
                if current_state.level_id == 1 && prev_state.level_id == 9 {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer for story mode...").as_str());
                    mode = Gamemode::CAREER;
                }
                // classic
                if current_state.level_id == 2 && prev_state.level_id == 0 && current_state.total_classic_goals == 0 {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer for classic mode...").as_str());
                    mode = Gamemode::CLASSIC;
                }
            },
            TimerState::Paused | TimerState::Running => {
                match mode {
                    Gamemode::NONE => {},
                    Gamemode::CAREER => {
                        if current_state.level_id != 0 && current_state.level_id != prev_state.level_id {
                            asr::timer::split();
                            asr::print_message(format!("Changed level; splitting timer...").as_str());
                        } 
                        
                        if current_state.is_game_finished && !prev_state.is_game_finished {
                            asr::timer::split();
                            asr::print_message(format!("Final cutscene; splitting timer...").as_str());
                        }
        
                        // reset when story start flag is unset
                        if current_state.level_id == 0 && !current_state.is_story_started && current_state.story_points == 0 {
                            asr::timer::reset();
                            asr::print_message(format!("Resetting timer...").as_str());
                            mode = Gamemode::NONE;
                        }
                    },
                    Gamemode::CLASSIC => {
                        if current_state.level_id != 0 && current_state.level_id != prev_state.level_id {
                            asr::timer::split();
                            asr::print_message(format!("Changed level; splitting timer...").as_str());
                        } 

                        if current_state.is_run_ended && !prev_state.is_run_ended && 
                            ((current_state.total_classic_goals < 120 && current_state.classic_difficulty == Difficulty::NORMAL && current_state.classic_triangle_goals >= 6) || 
                            (current_state.total_classic_goals < 120 && current_state.classic_difficulty == Difficulty::SICK && current_state.classic_triangle_goals >= 8) || 
                            (current_state.total_classic_goals == 140)) {
                            asr::timer::split();
                            asr::print_message(format!("End of classic mode; splitting timer...").as_str());
                        }

                        // reset when on 0 goals are completed
                        if current_state.level_id == 0 && prev_state.total_classic_goals == 0 {
                            asr::timer::reset();
                            asr::print_message(format!("Resetting timer...").as_str());
                            mode = Gamemode::NONE;
                        }
                    },
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