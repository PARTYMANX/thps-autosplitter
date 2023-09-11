use std::{collections::HashSet, rc::Rc, cell::RefCell};

use asr::{Address, Process, signature::Signature, string::ArrayCString, timer::TimerState};

// skate module, for me, is 0083e81c    a1 ?? ?? ?? ?? 85 c0 75 3d 8a 4c 24 04 84 c9 74 35 6a 01 50 68 9c 00 00 00
// level id: skate module -> 0x20 -> 0xb0
// flags are in skate module -> 0x20 -> down a ways
// cash: skate module -> 0x20 -> 0x21c
// goal manager: skate module -> 0x78
// HOW DO WE FIGURE THIS OUT?
// find where story goals are stored (maybe try finding flags)
// trace back what writes to flags
// 

struct Offsets {
    loading_screen: u32,
    last_cutscene: u32,
    skmodule: u32,
}

impl Offsets {
    pub fn get(process: &Process, base_addr: Address, module_size: u64) -> Self {
        let load_screen_ptr = Signature::<12>::new("a1 ?? ?? ?? ?? 73 05 b8 ?? ?? ?? ??").scan_process_range(process, (base_addr, module_size)).unwrap() + 1;
        let last_cutscene_ptr = Signature::<11>::new("74 15 ba ?? ?? ?? ?? 8b c6 2b d6").scan_process_range(process, (base_addr, module_size)).unwrap() + 3;
        let skmod_ptr = Signature::<25>::new("a1 ?? ?? ?? ?? 85 c0 75 3d 8a 4c 24 04 84 c9 74 35 6a 01 50 68 9c 00 00 00").scan_process_range(process, (base_addr, module_size)).unwrap() + 1;

        Offsets {
            loading_screen: match process.read::<u32>(load_screen_ptr) {
                Ok(v) => v - 0x400000,
                Err(_) => 0,
            },
            last_cutscene: match process.read::<u32>(last_cutscene_ptr) {
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

struct ScriptArray {
    _unk: u32,
    size: u32,
    pdata: u32,
}

impl ScriptArray {
    pub fn read(process: &Process, addr: u32) -> Self {
        Self {
            _unk: match process.read_pointer_path32::<u32>(addr, &vec!(0x0 as u32)) {
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

struct ScriptComponent {
    _unk: u8,
    ttype: u8,
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
            ttype: match process.read_pointer_path32::<u8>(addr, &vec!(0x1 as u32)) {
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

enum Gamemode {
    NONE,
    CAREER,
    CLASSIC,
}

struct Goal {
    mode: Gamemode,
    complete: bool,
    is_locked: bool,
    _unk1: u32,
    unk2: u32,
    _record: i32,
}

fn read_goal(process: &Process, addr: u32) -> Option<Goal> {
    let stru = ScriptStruct::read(process, addr);
    let mut comp = stru.phead;

    let mut mode = Gamemode::NONE;
    let mut complete = false;
    let mut unlocked = false;
    let mut _unk1 = 0;
    let mut unk2 = 0;
    let mut _record = 0;

    while comp != 0 {
        let component = ScriptComponent::read(process, comp);
        //asr::print_message(format!("COMPONENT {:02}: type: {:#04x} name: {:#010x} data: {:#010x} next: {:#010x}", counter, component.ttype >> 1, component.name, component.data, component.pnext).as_str());

        if component.name == 0x49807745 {   // "hasBeaten"
            if component.data == 1 {
                complete = true;
            }
        } else if component.name == 0xc22a2b72 {
            _record = component.data as i32;
        } else if component.name == 0xd3e93882 {
            if component.data == 1 {
                unlocked = true;
            }
        } else if component.name == 0x2206b1e7 {
            _unk1 = component.data;
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
            //asr::print_message(&format!("UNKNOWN COMPONENT {:#010x}, {}, {:#010x}", component.name, component.ttype, component.data));
        }

        comp = component.pnext;
    }

    if !matches!(mode, Gamemode::NONE) {
        Some (Goal {
            mode,
            complete,
            is_locked: unlocked,
            _unk1,
            unk2,
            _record,
        })
    } else {
        None
    }
}

 #[derive(Debug, PartialEq, Eq)] 
enum Difficulty {
    UNKNOWN,
    EASY,
    NORMAL,
    SICK,
}

fn read_difficulty_levels(process: &Process, addr: u32, state: &mut State) {
    let stru = ScriptStruct::read(process, addr);
    let mut comp = stru.phead;
    while comp != 0 {
        let component = ScriptComponent::read(process, comp);
        //asr::print_message(format!("COMPONENT {:02}: type: {:#04x} name: {:#010x} data: {:#010x} next: {:#010x}", counter, component.ttype >> 1, component.name, component.data, component.pnext).as_str());

        if component.name == 0x4da4937b {   // "career"
            if component.data == 0 {
                state.story_difficulty = Difficulty::EASY;
            } else if component.data == 1 {
                state.story_difficulty = Difficulty::NORMAL;
            } else if component.data == 2 {
                state.story_difficulty = Difficulty::SICK;
            }
        } else if component.name == 0x4C10DE52 {    // "classic"
            if component.data == 0 {
                state.classic_difficulty = Difficulty::NORMAL;
            } else if component.data == 1 {
                state.classic_difficulty = Difficulty::SICK;
            }
        }

        comp = component.pnext;
    }
}

fn read_goal_params(process: &Process, addr: u32, state: &mut State) {
    let stru = ScriptStruct::read(process, addr);
    let mut comp = stru.phead;
    while comp != 0 {
        let component = ScriptComponent::read(process, comp);
        //asr::print_message(format!("COMPONENT {:02}: type: {:#04x} name: {:#010x} data: {:#010x} next: {:#010x}", counter, component.ttype >> 1, component.name, component.data, component.pnext).as_str());

        if component.name == 0xb13d98d3 {   // "difficulty_levels"
            read_difficulty_levels(process, component.data, state);
        }

        comp = component.pnext;
    }
}

fn update_goal_flags(process: &Process, addr: u32, state: &mut State) {
    let stru = ScriptStruct::read(process, addr);
    let mut comp = stru.phead;
    while comp != 0 {
        let component = ScriptComponent::read(process, comp);
        //asr::print_message(format!("COMPONENT {:02}: type: {:#04x} name: {:#010x} data: {:#010x} next: {:#010x}", counter, component.ttype >> 1, component.name, component.data, component.pnext).as_str());

        if component.name == 0x23d4170a {   // "goalmanager_params"
            read_goal_params(process, component.data, state);
        } else if component.ttype >> 1 == 0xa {
            if let Some(goal) = read_goal(process, component.data) {
                if component.name == 0x55c9d19c {
                    //asr::print_message(&format!("Goal {:#010x}: unk1: {} unk2: {} locked: {} complete: {} record: {}", component.name, goal._unk1, goal.unk2, goal.is_locked, goal.complete, goal._record));
                }
                
                if goal.complete && goal.unk2 == 1 && state.completed_goals.borrow_mut().insert(component.name) {
                    if matches!(goal.mode, Gamemode::CAREER) {
                        state.story_goals += 1;
                    } else {
                        state.classic_goals += 1;
                    }
                }

                if goal.is_locked {
                    state.locked_goals.borrow_mut().insert(component.name);
                    //asr::print_message(&format!("GOAL {:#010x} IS NOT UNLOCKED", component.name));
                }
            }
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
    _ptail: u32,
    _pcurrent: u32,
    _idx: u32,
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
            _ptail: match process.read_pointer_path32::<u32>(addr, &vec!(0x8 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            _pcurrent: match process.read_pointer_path32::<u32>(addr, &vec!(0xc as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
            _idx: match process.read_pointer_path32::<u32>(addr, &vec!(0x10 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
        }
    }
}

static CLASSIC_LEVELS: [u32; 6] = [
    11, // minneapolis
    15, // santa cruz
    13, // mall
    10, // chicago
    12, // kyoto
    17, // the ruins
];

fn is_classic_level(id: u32) -> bool {
    CLASSIC_LEVELS.contains(&id)
}

fn update_goals(process: &Process, addr: u32, state: &mut State) {
    let goal_list = GoalList::read(process, addr);

    let mut pnode = goal_list.phead;
    for _ in 0..goal_list.num {
        let node = GoalListNode::read(process, pnode);
        if let Ok(flags) = process.read_pointer_path32::<u32>(node.pgoal, &vec!(0x68 as u32)) {
            /*if !goalflags.contains_key(&node.name) {
                goalflags.insert(node.name, flags | 0x0400);
                asr::print_message(&format!("READING {} GOALS", goal_list.num));
                asr::print_message(&format!("GOAL {:#010x} FLAGS: {:#010x}", node.name, flags));
            } else {
                if let Some(f) = goalflags.get_mut(&node.name) {
                    if *f & 0x20 != 0 && flags & 0x20 == 0 {
                        asr::print_message(&format!("GOAL {:#010x} UNLOCKED", node.name));
                    }

                    if *f != flags | 0x0400 {
                        asr::print_message(&format!("GOAL {:#010x} FLAGS CHANGED FROM: {:#010x} TO: {:#010x}", node.name, f, flags));
                        *f = flags | 0x0400;
                    }
                }
            }*/

            if flags & 0x4 != 0 {
                if !state.completed_goals.borrow().contains(&node.name) {
                    let mut is_leaf = false;

                    if let Ok(parent) = process.read_pointer_path32::<u32>(node.pgoal, &vec!(0x28 as u32, 0x0 as u32))  {
                        if parent == 0 {
                            state.completed_goals.borrow_mut().insert(node.name);
                            is_leaf = true;
                        }
                    } else {
                        state.completed_goals.borrow_mut().insert(node.name);
                        is_leaf = true;
                    }

                    if is_leaf {
                        if !is_classic_level(state.level_id) {
                            state.story_goals += 1;
                        } else {
                            state.classic_goals += 1;
                        }
                    }
                }
            } else if flags & 0x20 != 0 {
                state.locked_goals.borrow_mut().insert(node.name);
            }

            if flags & 0x100 != 0 {
                state.run_is_active = true;
            }
        }

        pnode = node.pnext;
    }
}

struct State {
    level_id: u32,
    loading_screen: String,
    last_cutscene: String,
    story_goals: u32,
    classic_goals: u32,
    completed_goals: Rc<RefCell<HashSet<u32>>>,
    locked_goals: Rc<RefCell<HashSet<u32>>>,
    run_is_active: bool,
    session_goals: u32,
    story_difficulty: Difficulty,
    classic_difficulty: Difficulty,
}

impl State {
    pub fn update(process: &Process, base_addr: Address, offsets: &Offsets) -> Self {
        let mut result = Self {
            level_id: match process.read_pointer_path32::<u32>(base_addr, &vec!(offsets.skmodule as u32, 0x20 as u32, 0xb0 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
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

            story_goals: 0,
            classic_goals: 0,
            run_is_active: false,
            completed_goals: Rc::new(RefCell::new(HashSet::new())),
            locked_goals: Rc::new(RefCell::new(HashSet::new())),
            classic_difficulty: Difficulty::UNKNOWN,
            story_difficulty: Difficulty::UNKNOWN,

            session_goals: match process.read_pointer_path32::<u32>(base_addr, &vec!(offsets.skmodule as u32, 0x78 as u32, 0x38 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },
        };

        if let Ok(addr) = process.read_pointer_path32::<u32>(base_addr, &vec!(offsets.skmodule as u32, 0x78 as u32)) {
            update_goals(process, addr, &mut result);
        }

        if let Ok(addr) = process.read_pointer_path32::<u32>(base_addr, &vec!(offsets.skmodule as u32, 0x78 as u32, 0x14 as u32)) {
            update_goal_flags(process, addr, &mut result);
        }

        return result;
    }
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THAW!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();
    let module_size = process.get_module_size(process_name).unwrap();

    let offsets = Offsets::get(process, base_addr, module_size);

    let mut prev_state = State::update(process, base_addr, &offsets);

    loop {
        // update vars
        let mut current_state = State::update(process, base_addr, &offsets);

        //dump_career_struct(process, base_addr, &offsets);
        //dump_career_list(process, base_addr, &offsets);
        // mystery career struct
        //if let Ok(addr) = process.read_pointer_path32::<u32>(base_addr, &vec!(offsets.skmodule as u32, 0x20 as u32, 0x254 as u32)) {
        //    dump_struct(process, addr);
        //}

        // sometimes the read of goal flags fails and causes a bunch of goals to re-complete  
        // to avoid that, just copy previous progress if session goals hasn't changed but the others have
        if (current_state.story_goals != prev_state.story_goals || 
            current_state.classic_goals != prev_state.classic_goals) &&
            current_state.session_goals == prev_state.session_goals {

            asr::print_message(&format!("WARNING: goal flags failed to read!"));
            
            for i in prev_state.completed_goals.borrow().iter() {
                current_state.completed_goals.borrow_mut().insert(*i);
            }
            for i in prev_state.locked_goals.borrow().iter() {
                current_state.locked_goals.borrow_mut().insert(*i);
            }
            //current_state.story_goals = prev_state.story_goals;
            //current_state.classic_goals = prev_state.classic_goals;

            //continue;
        }

        for i in current_state.completed_goals.borrow().iter() {
            if !prev_state.completed_goals.borrow().contains(&i) {
                asr::print_message(&format!("COMPLETED GOAL {:#010x}!", i));
            }
        }

        if current_state.level_id == prev_state.level_id && current_state.story_goals >= prev_state.story_goals {
            for i in prev_state.locked_goals.borrow().iter() {
                if !current_state.locked_goals.borrow().contains(&i) {
                    asr::print_message(&format!("UNLOCKED GOAL {:#010x}!", i));
                }
            }
        }

        // pause game time when loading, resume when done
        /*if current_state.is_loading && !prev_state.is_loading {
            asr::timer::pause_game_time();
            asr::print_message(format!("Starting Load...").as_str());
        } else if !current_state.is_loading && prev_state.is_loading {
            asr::timer::resume_game_time();
            asr::print_message(format!("Done Loading").as_str());
        }*/

        if current_state.level_id != prev_state.level_id {
            asr::print_message(format!("Level changed to {}!", current_state.level_id).as_str());
        }
        if current_state.last_cutscene != prev_state.last_cutscene {
            asr::print_message(format!("Last cutscene changed to {}!", current_state.last_cutscene).as_str());
        }
        if current_state.loading_screen != prev_state.loading_screen {
            asr::print_message(format!("Loading screen changed to {}!", current_state.loading_screen).as_str());
        }
        if current_state.story_goals != prev_state.story_goals {
            asr::print_message(format!("Story goals changed to {}!", current_state.story_goals).as_str());
        }
        if current_state.classic_goals != prev_state.classic_goals {
            asr::print_message(format!("Classic goals changed to {}!", current_state.classic_goals).as_str());
        }
        if current_state.run_is_active != prev_state.run_is_active {
            asr::print_message(format!("Run active state changed to {}!", current_state.run_is_active).as_str());
        }
        if current_state.story_difficulty != prev_state.story_difficulty {
            asr::print_message(format!("Story difficulty changed to {:?}!", current_state.story_difficulty).as_str());
        }
        if current_state.classic_difficulty != prev_state.classic_difficulty {
            asr::print_message(format!("Classic difficulty changed to {:?}!", current_state.classic_difficulty).as_str());
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