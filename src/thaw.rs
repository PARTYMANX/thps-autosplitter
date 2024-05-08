use std::{collections::HashSet, rc::Rc, cell::RefCell};

use asr::{Address, Process, signature::Signature, timer::TimerState};

struct Offsets {
    skmodule: u32,
    load_counter: u32,
}

impl Offsets {
    pub fn get(process: &Process, base_addr: Address, module_size: u64) -> Self {
        let skmod_ptr = Signature::<25>::new("a1 ?? ?? ?? ?? 85 c0 75 3d 8a 4c 24 04 84 c9 74 35 6a 01 50 68 9c 00 00 00").scan_process_range(process, (base_addr, module_size)).unwrap() + 1;
        let load_counter_ptr = Signature::<18>::new("84 c0 75 05 e8 ?? ?? ?? ?? a1 ?? ?? ?? ?? 85 c0 74 13").scan_process_range(process, (base_addr, module_size)).unwrap() + 10;

        Offsets {
            skmodule: match process.read::<u32>(skmod_ptr) {
                Ok(v) => v - 0x400000,
                Err(_) => 0,
            },
            load_counter: match process.read::<u32>(load_counter_ptr) {
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

/*
// keeping this here in case it's needed for collectors edition (to check stats)...
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
*/

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
    _is_locked: bool,
    _unk1: u32,
    unk2: u32,
    _record: i32,
}

fn read_goal(process: &Process, addr: u32) -> Option<Goal> {
    let stru = ScriptStruct::read(process, addr);
    let mut comp = stru.phead;

    let mut mode = Gamemode::NONE;
    let mut complete = false;
    let mut _is_locked = false;
    let mut _unk1 = 0;
    let mut unk2 = 0;
    let mut _record = 0;

    while comp != 0 {
        let component = ScriptComponent::read(process, comp);

        if component.name == 0x49807745 {   // "hasBeaten"
            if component.data == 1 {
                complete = true;
            }
        } else if component.name == 0xc22a2b72 {
            _record = component.data as i32;
        } else if component.name == 0xd3e93882 {
            if component.data == 1 {
                _is_locked = true;
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
        }

        comp = component.pnext;
    }

    if !matches!(mode, Gamemode::NONE) {
        Some (Goal {
            mode,
            complete,
            _is_locked,
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

        if component.name == 0x23d4170a {   // "goalmanager_params"
            read_goal_params(process, component.data, state);
        } else if component.ttype >> 1 == 0xa {
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
            } //else if flags & 0x20 != 0 {
            //    state.locked_goals.borrow_mut().insert(node.name);
            //}

            if flags & 0x100 != 0 {
                state.run_is_active = true;
            }
        }

        pnode = node.pnext;
    }
}

struct State {
    level_id: u32,
    load_counter: u32,
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

            load_counter: match process.read_pointer_path32::<u32>(base_addr, &vec!(offsets.load_counter as u32)) {
                Ok(v) => v,
                Err(_) => 0,
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

static RANCH_GOALS: [u32; 3] = [
    0x7c626a8a, // boone
    0xe206ff29, // dave
    0x0b655a1c, // murphy
];

static CASINO_GOALS: [u32; 7] = [
    0x8c0507e5,
    0x419fe6b0,
    0x9b9cd093,
    0x4829aa9b,
    0x3f2e9a0d,
    0xd120fb21,
    0xd67b166d,
];

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THAW!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();
    let module_size = process.get_module_size(process_name).unwrap();

    let offsets = Offsets::get(process, base_addr, module_size);

    let mut prev_state = State::update(process, base_addr, &offsets);

    let mut is_paused = false;
    let mut mode = Gamemode::NONE;
    let mut story_flags = [
        false, // beverly hills opened OR beverly hills visited(?) (complete 0x7a446a0a OR level == 2)
        false, // ranch unlocked (finished challenges)
        false, // downtown unlocked (falling down goal) (complete 0x8f0cd51c)
        false, // amjam unlocked (last zen goal) (complete 0x607572ca)
        false, // amjam complete (amjam goal) (complete 0x3a800176)
        false, // santa monica unlocked (beat daewon) (complete 0xb9415865)
        false, // oil rig unlocked (paid oil rig) (complete 0xea9af00c)
        false, // oil rig route unlocked (chopper goal) (complete 0xd32df1bb)
        false, // east la unlocked (black widowz goal) (complete 0xda77ca92)
        false, // pro goals started (joey b baggie) (complete 0x97e3e2cf)
        false, // casino unlocked (paid taco truck) (complete 0x82510d90)
        false, // final goal unlocked (completed x casino goals) 
        false, // final goal complete (complete 0x99156422)
    ];

    loop {
        // update vars
        let current_state = State::update(process, base_addr, &offsets);

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
        }

        let mut completed_goals = Vec::new();
        for i in current_state.completed_goals.borrow().iter() {
            if !prev_state.completed_goals.borrow().contains(&i) {
                completed_goals.push(*i);
            }
        }

        /*
        if current_state.level_id == prev_state.level_id && current_state.story_goals >= prev_state.story_goals {
            for i in prev_state.locked_goals.borrow().iter() {
                if !current_state.locked_goals.borrow().contains(&i) {
                    asr::print_message(&format!("UNLOCKED GOAL {:#010x}!", i));
                }
            }
        }
        */

        match asr::timer::state() {
            TimerState::NotRunning => {
                story_flags.fill(false);

                if is_paused {
                    is_paused = false;
                    asr::timer::resume_game_time();
                    asr::print_message(format!("Done loading!").as_str());
                }

                // start story when level == 1, no goals are complete, and goal is active
                if current_state.level_id == 1 && current_state.story_goals == 0 && current_state.run_is_active {
                    asr::timer::start();
                    asr::print_message(&format!("Starting timer for Story..."));
                    mode = Gamemode::CAREER;
                }

                // start classic when level == 11 from menu
                if current_state.level_id == 11 && prev_state.level_id == 0 && current_state.classic_goals == 0 {
                    asr::timer::start();
                    asr::print_message(&format!("Starting timer for Classic..."));
                    mode = Gamemode::CLASSIC;
                }
            },
            TimerState::Paused | TimerState::Running => {
                match mode {
                    Gamemode::NONE => {},
                    Gamemode::CAREER => {
                        if is_paused {
                            is_paused = false;
                            asr::timer::resume_game_time();
                            asr::print_message(format!("Done loading!").as_str());
                        }

                        // split on unlock or visit beverly hills
                        if !story_flags[0] && (completed_goals.contains(&0x7a446a0a) || (current_state.level_id == 2 && prev_state.level_id != 2)) {
                            asr::timer::split();
                            asr::print_message(format!("Unlocked or skipped to Beverly Hills; splitting timer...").as_str());
                            story_flags[0] = true;
                        }

                        // split on unlock or visit beverly hills
                        if !story_flags[1] {
                            // calculate ranch unlock conditions
                            let mut ranch_conditions = 0;
                            ranch_conditions += match current_state.story_difficulty {
                                Difficulty::UNKNOWN => 0,
                                Difficulty::EASY => 2,
                                Difficulty::NORMAL => 1,
                                Difficulty::SICK => 0,
                            };

                            for i in RANCH_GOALS {
                                if current_state.completed_goals.borrow().contains(&i) {
                                    ranch_conditions += 1;
                                }
                            }

                            if ranch_conditions >= 3 {
                                asr::timer::split();
                                asr::print_message(format!("Unlocked Skate Ranch; splitting timer...").as_str());
                                story_flags[1] = true;
                            }
                        }

                        // split on unlock downtown
                        if !story_flags[2] && completed_goals.contains(&0x8f0cd51c) {
                            asr::timer::split();
                            asr::print_message(format!("Unlocked Downtown; splitting timer...").as_str());
                            story_flags[2] = true;
                        }

                        // split on unlock amjam
                        if !story_flags[3] && completed_goals.contains(&0x607572ca) {
                            asr::timer::split();
                            asr::print_message(format!("Unlocked Amjam; splitting timer...").as_str());
                            story_flags[3] = true;
                        }

                        // split on win amjam
                        if !story_flags[4] && completed_goals.contains(&0x3a800176) {
                            asr::timer::split();
                            asr::print_message(format!("Won Amjam; splitting timer...").as_str());
                            story_flags[4] = true;
                        }

                        // split on unlock santa monica
                        if !story_flags[5] && completed_goals.contains(&0xb9415865) {
                            asr::timer::split();
                            asr::print_message(format!("Unlocked Santa Monica; splitting timer...").as_str());
                            story_flags[5] = true;
                        }

                        // split on unlock oil rig
                        if !story_flags[6] && completed_goals.contains(&0xea9af00c) {
                            asr::timer::split();
                            asr::print_message(format!("Unlocked Oil Rig; splitting timer...").as_str());
                            story_flags[6] = true;
                        }

                        // split on finish oil rig
                        if !story_flags[7] && completed_goals.contains(&0xd32df1bb) {
                            asr::timer::split();
                            asr::print_message(format!("Finished Oil Rig; splitting timer...").as_str());
                            story_flags[7] = true;
                        }

                        // split on unlock east la
                        if !story_flags[8] && completed_goals.contains(&0xda77ca92) {
                            asr::timer::split();
                            asr::print_message(format!("Unlocked East LA; splitting timer...").as_str());
                            story_flags[8] = true;
                        }

                        // split on unlock santa monica
                        if !story_flags[9] && completed_goals.contains(&0x97e3e2cf) {
                            asr::timer::split();
                            asr::print_message(format!("Started Pro Goals; splitting timer...").as_str());
                            story_flags[9] = true;
                        }

                        // split on unlock casino
                        if !story_flags[10] && completed_goals.contains(&0x82510d90) {
                            asr::timer::split();
                            asr::print_message(format!("Unlocked Casino; splitting timer...").as_str());
                            story_flags[10] = true;
                        }

                        // split on unlock final goal
                        if !story_flags[11] {
                            // calculate ranch unlock conditions
                            let mut unlock_conditions = 0;
                            unlock_conditions += match current_state.story_difficulty {
                                Difficulty::UNKNOWN => 0,
                                Difficulty::EASY => 3,
                                Difficulty::NORMAL => 2,
                                Difficulty::SICK => 0,
                            };

                            for i in CASINO_GOALS {
                                if current_state.completed_goals.borrow().contains(&i) {
                                    unlock_conditions += 1;
                                }
                            }

                            if unlock_conditions >= 7 {
                                asr::timer::split();
                                asr::print_message(format!("Unlocked final goal; splitting timer...").as_str());
                                story_flags[11] = true;
                            }
                        }

                        // split on final goal
                        if !story_flags[12] && completed_goals.contains(&0x99156422) {
                            asr::timer::split();
                            asr::print_message(format!("Completed final goal; splitting timer...").as_str());
                            story_flags[12] = true;
                        }

                        // reset on 0 goals complete on menu
                        if current_state.level_id == 0 && current_state.story_goals == 0 {
                            asr::timer::reset();
                            asr::print_message(format!("Resetting timer...").as_str());
                            story_flags.fill(false);
                        }
                    },
                    Gamemode::CLASSIC => {
                        if current_state.load_counter > 0 && !is_paused {
                            is_paused = true;
                            asr::timer::pause_game_time();
                            asr::print_message(format!("Starting Load... {}", current_state.load_counter).as_str());
                        } else if current_state.load_counter == 0 && is_paused {
                            is_paused = false;
                            asr::timer::resume_game_time();
                            asr::print_message(format!("Done loading!").as_str());
                        }

                        // split on level changes (except main menu)
                        if current_state.level_id != prev_state.level_id && current_state.level_id != 0 {
                            asr::timer::split();
                            asr::print_message(format!("Changed level; splitting timer...").as_str());
                        }

                        // split on end run when 51 goals are complete
                        if current_state.classic_goals >= 51 && !current_state.run_is_active && prev_state.run_is_active {  // FIXME: goes off when restarting
                            asr::timer::split();
                            asr::print_message(format!("Classic complete; splitting timer...").as_str());
                        }

                        // reset on 0 goals complete on menu
                        if current_state.level_id == 0 && current_state.classic_goals == 0 {
                            asr::timer::reset();
                            asr::print_message(format!("Resetting timer...").as_str());
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