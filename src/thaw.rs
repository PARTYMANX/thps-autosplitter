use std::{path::Component, collections::{HashSet, HashMap}};

use asr::{Address, Process, signature::Signature, string::ArrayCString, timer::TimerState, Address32};

// skate module, for me, is 0083e81c    a1 ?? ?? ?? ?? 85 c0 75 3d 8a 4c 24 04 84 c9 74 35 6a 01 50 68 9c 00 00 00
// level id: skate module -> 0x20 -> 0xb0
// flags are in skate module -> 0x20 -> down a ways
// cash: skate module -> 0x20 -> 0x21c
// goal manager: skate module -> 0x78
// HOW DO WE FIGURE THIS OUT?
// find where story goals are stored (maybe try finding flags)
// trace back what writes to flags
// 

struct scrStruct {
    unk: u32,
    phead: u32,
}

impl scrStruct {
    pub fn read(process: &Process, addr: u32) -> Self {
        Self {
            unk: match process.read_pointer_path32::<u32>(addr, &vec!(0x0 as u32)) {
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

struct scrArray {
    unk: u32,
    size: u32,
    pdata: u32,
}

impl scrArray {
    pub fn read(process: &Process, addr: u32) -> Self {
        Self {
            unk: match process.read_pointer_path32::<u32>(addr, &vec!(0x0 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            size: match process.read_pointer_path32::<u32>(addr, &vec!(0x4 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            pdata: match process.read_pointer_path32::<u32>(addr, &vec!(0x8 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
        }
    }
}

struct scrComponent {
    unk: u8,
    ttype: u8,
    size: u16,
    name: u32,
    data: u32,
    pnext: u32,
}

impl scrComponent {
    pub fn read(process: &Process, addr: u32) -> Self {
        Self {
            unk: match process.read_pointer_path32::<u8>(addr, &vec!(0x0 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            ttype: match process.read_pointer_path32::<u8>(addr, &vec!(0x1 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            size: match process.read_pointer_path32::<u16>(addr, &vec!(0x2 as u32)) {
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

enum Gamemode {
    NONE,
    CAREER,
    CLASSIC,
}

struct Goal {
    mode: Gamemode,
    complete: bool,
    unlocked: bool,
    unk1: u32,
    unk2: u32,
    record: i32,
}

fn read_goal(process: &Process, addr: u32) -> Option<Goal> {
    let stru = scrStruct::read(process, addr);
    let mut comp = stru.phead;

    let mut mode = Gamemode::NONE;
    let mut complete = false;
    let mut unlocked = false;
    let mut unk1 = 0;
    let mut unk2 = 0;
    let mut record = 0;

    while comp != 0 {
        let component = scrComponent::read(process, comp);
        //asr::print_message(format!("COMPONENT {:02}: type: {:#04x} name: {:#010x} data: {:#010x} next: {:#010x}", counter, component.ttype >> 1, component.name, component.data, component.pnext).as_str());

        if component.name == 0x49807745 {   // "hasBeaten"
            if component.data == 1 {
                complete = true;
            }
        } else if component.name == 0xc22a2b72 {
            record = component.data as i32;
        } else if component.name == 0xd3e93882 {
            if component.data == 1 {
                unlocked = true;
            }
        } else if component.name == 0x2206b1e7 {
            unk1 = component.data;
        } else if component.name == 0x290a18e3 {
            unk2 = component.data;
        } else if component.ttype >> 1 == 0xd {
            if component.data == 0x4da4937b {
                mode = Gamemode::CAREER;
            }
            if component.data == 0x4C10DE52 {
                mode = Gamemode::CLASSIC;
            }
        } else {
            asr::print_message(&format!("UNKNOWN COMPONENT {:#010x}", component.name));
        }

        comp = component.pnext;
    }

    if !matches!(mode, Gamemode::NONE) {
        Some (Goal {
            mode,
            complete,
            unlocked,
            unk1,
            unk2,
            record,
        })
    } else {
        None
    }
}

fn read_goal_params(process: &Process, addr: u32) {
    let stru = scrStruct::read(process, addr);
    let mut comp = stru.phead;
    while comp != 0 {
        let component = scrComponent::read(process, comp);
        //asr::print_message(format!("COMPONENT {:02}: type: {:#04x} name: {:#010x} data: {:#010x} next: {:#010x}", counter, component.ttype >> 1, component.name, component.data, component.pnext).as_str());

        if component.name == 0x49807745 {   // "difficulty_levels"
            // do nothing
        }

        comp = component.pnext;
    }
}

fn update_goal_flags(process: &Process, addr: u32, goals: &mut HashSet<u32>) {
    let stru = scrStruct::read(process, addr);
    let mut comp = stru.phead;
    while comp != 0 {
        let component = scrComponent::read(process, comp);
        //asr::print_message(format!("COMPONENT {:02}: type: {:#04x} name: {:#010x} data: {:#010x} next: {:#010x}", counter, component.ttype >> 1, component.name, component.data, component.pnext).as_str());

        if component.name == 0x23d4170a {   // "goalmanager_params"
            read_goal_params(process, component.data);
        } else if component.ttype >> 1 == 0xa {
            if let Some(goal) = read_goal(process, component.data) {
                //asr::print_message(&format!("Goal {:#010x}: unk1: {} unk2: {} unlocked: {} complete: {} record: {}", component.name, goal.unk1, goal.unk2, goal.unlocked, goal.complete, goal.record));
                if goal.complete && goal.unk2 == 1 && !goals.contains(&component.name) {
                    goals.insert(component.name);
                    asr::print_message(&format!("COMPLETED GOAL {:#010x} {}", component.name, goals.len()));
                }
            } else {
                //asr::print_message(&format!("GOAL MANAGER UNKNOWN COMPONENT AAAA: {:#010x}", component.name));
            }
        } else {
            //asr::print_message(&format!("GOAL MANAGER UNKNOWN COMPONENT: {:#010x}, type: {}", component.name, component.ttype >> 1));
        }

        comp = component.pnext;
    }
}

struct GoalListNode {
    name: u32,
    pgoal: u32,
    pnext: u32,
}

impl GoalListNode {
    pub fn read(process: &Process, addr: u32) -> Self {
        Self {
            name: match process.read_pointer_path32::<u32>(addr, &vec!(0x0 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            pgoal: match process.read_pointer_path32::<u32>(addr, &vec!(0x4 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            pnext: match process.read_pointer_path32::<u32>(addr, &vec!(0x8 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
        }
    }
}

struct GoalList {
    num: u32,
    phead: u32,
    ptail: u32,
    pcurrent: u32,
    idx: u32,
}

impl GoalList {
    pub fn read(process: &Process, addr: u32) -> Self {
        Self {
            num: match process.read_pointer_path32::<u32>(addr, &vec!(0x0 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            phead: match process.read_pointer_path32::<u32>(addr, &vec!(0x4 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            ptail: match process.read_pointer_path32::<u32>(addr, &vec!(0x8 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            pcurrent: match process.read_pointer_path32::<u32>(addr, &vec!(0xc as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            idx: match process.read_pointer_path32::<u32>(addr, &vec!(0x10 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
        }
    }
}

fn read_goals(process: &Process, addr: u32, goals: &mut HashSet<u32>, goalflags: &mut HashMap<u32, u32>) {
    let goal_list = GoalList::read(process, addr);

    //asr::print_message(&format!("READING {} GOALS", goal_list.num));

    let mut pnode = goal_list.phead;
    for i in 0..goal_list.num {
        let node = GoalListNode::read(process, pnode);

        if !goals.contains(&node.name) {
            if let Ok(flags) = process.read_pointer_path32::<u32>(node.pgoal, &vec!(0x68 as u32)) {
                if !goalflags.contains_key(&node.name) {
                    goalflags.insert(node.name, flags | 0x0400);
                    //asr::print_message(&format!("GOAL {:#010x} FLAGS: {:#010x}", node.name, flags));
                } else {
                    if let Some(f) = goalflags.get_mut(&node.name) {
                        if *f != flags | 0x0400 {
                            //asr::print_message(&format!("GOAL {:#010x} FLAGS CHANGED FROM: {:#010x} TO: {:#010x}", node.name, f, flags));
                            *f = flags | 0x0400;
                        }
                    }
                }

                if flags & 0x4 != 0 {
                    //goals.insert(node.name);
                    
                    if let Ok(parent) = process.read_pointer_path32::<u32>(node.pgoal, &vec!(0x28 as u32, 0x0 as u32))  {
                        if parent == 0 {
                            goals.insert(node.name);
                            asr::print_message(&format!("COMPLETED GOAL {:#010x}, {}", node.name, goals.len()));
                        }
                    } else {
                        asr::print_message(&format!("COMPLETED GOAL {:#010x} {}", node.name, goals.len()));
                        goals.insert(node.name);
                    }
                }
                
            }
        }

        pnode = node.pnext;
    }
}


struct Offsets {
    level_name: u32,
    loading_screen: u32,
    last_cutscene: u32,
    total_goals: u32,
    is_run_completed: u32,
    skmodule: u32,
}

impl Offsets {
    pub fn get(process: &Process, base_addr: Address, module_size: u64) -> Self {
        let level_name_ptr = Signature::<10>::new("8b 0c 24 51 68 ?? ?? ?? ?? e8").scan_process_range(process, (base_addr, module_size)).unwrap() + 5;
        let load_screen_ptr = Signature::<12>::new("a1 ?? ?? ?? ?? 73 05 b8 ?? ?? ?? ??").scan_process_range(process, (base_addr, module_size)).unwrap() + 1;
        let last_cutscene_ptr = Signature::<11>::new("74 15 ba ?? ?? ?? ?? 8b c6 2b d6").scan_process_range(process, (base_addr, module_size)).unwrap() + 3;
        let total_goals_ptr = Signature::<12>::new("39 5c 24 24 74 10 8b 0d ?? ?? ?? ??").scan_process_range(process, (base_addr, module_size)).unwrap() + 8;
        let is_run_completed_ptr = Signature::<11>::new("8b 04 85 ?? ?? ?? ?? 5e c2 04 00").scan_process_range(process, (base_addr, module_size)).unwrap() + 3;
        let skmod_ptr = Signature::<25>::new("a1 ?? ?? ?? ?? 85 c0 75 3d 8a 4c 24 04 84 c9 74 35 6a 01 50 68 9c 00 00 00").scan_process_range(process, (base_addr, module_size)).unwrap() + 1;

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
            skmodule: match process.read::<u32>(skmod_ptr) {
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
    session_goals: u32,
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

            session_goals: match process.read_pointer_path32::<u32>(base_addr, &vec!(offsets.skmodule as u32, 0x78 as u32, 0x38 as u32)) {
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
    let mut goals = HashSet::new();
    let mut goalflags = HashMap::new();

    loop {
        // update vars
        let current_state = State::update(process, base_addr, &offsets);

        //dump_career_struct(process, base_addr, &offsets);
        //dump_career_list(process, base_addr, &offsets);
        // mystery career struct
        //if let Ok(addr) = process.read_pointer_path32::<u32>(base_addr, &vec!(offsets.skmodule as u32, 0x20 as u32, 0x254 as u32)) {
        //    dump_struct(process, addr);
        //}

        // goalmanager struct
        if current_state.session_goals < prev_state.session_goals {
            goals.clear();
            goalflags.clear();
        }

        if let Ok(addr) = process.read_pointer_path32::<u32>(base_addr, &vec!(offsets.skmodule as u32, 0x78 as u32, 0x14 as u32)) {
            update_goal_flags(process, addr, &mut goals);
        }

        if let Ok(addr) = process.read_pointer_path32::<u32>(base_addr, &vec!(offsets.skmodule as u32, 0x78 as u32)) {
            read_goals(process, addr, &mut goals, &mut goalflags);
        }

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