use std::collections::HashMap;

use asr::{game_engine::unreal::FNameKey, timer::TimerState, Address, Address64, Process};
use once_cell::sync::Lazy;

fn get_fname_string(process: &Process, module: &asr::game_engine::unreal::Module, key: asr::game_engine::unreal::FNameKey) -> String {
    // if the key is null, return an empty string, otherwise we get "None"
    if key.is_null() {
        return "".to_string()
    }

    match module.get_fname::<256>(process, key) {
        Ok(v) => {
            match v.validate_utf8() {
                Ok(v) => v.to_string(),
                Err(_) => "".to_string(),
            }
        },
        Err(_) => "".to_string(),
    }
}

fn get_goal_system_pointer(process: &Process, module: &asr::game_engine::unreal::Module) -> Result<asr::Address64, asr::Error> {
    // find LocalPlayerGoalSystem
    let subsystem_count = match process.read_pointer_path::<u32>(
        module.g_world(), 
        asr::PointerSize::Bit64, 
        &vec!(
            0x0 as u64, 
            0x180 as u64, // GameInstance
            0x38 as u64, // LocalPlayers
            0x0 as u64, // Index first array index to get the AlcatrazLocalPlayer instance (maybe make this safer?)
            0xf0 as u64, // Subsystems I think?
            //0x98 as u64, // LocalPlayerGoalSystem
            //0x30 as u64, // GoalSystem
        ) 
    ) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    //asr::print_message(&format!("Subsystem count: {}", subsystem_count));

    let mut goal_system_idx = None;

    for i in 0..subsystem_count {
        let key = match process.read_pointer_path::<FNameKey>(
            module.g_world(), 
            asr::PointerSize::Bit64, 
            &vec!(
                0x0 as u64, 
                0x180 as u64, // GameInstance
                0x38 as u64, // LocalPlayers
                0x0 as u64, // Index first array index to get the AlcatrazLocalPlayer instance (maybe make this safer?)
                0xe8 as u64, // Subsystems I think?
                (i * 24) as u64 + 8 as u64, // index into array
                0x18 as u64, // get fname key
                //0x98 as u64, // LocalPlayerGoalSystem
                //0x30 as u64, // GoalSystem
            ) 
        ) {
            Ok(v) => v,
            Err(_) => FNameKey::default(),
        };

        let name = get_fname_string(process, module, key);

        //asr::print_message(&format!("    {}: {}", i, name));

        if name == "LocalPlayerGoalSystem" {
            goal_system_idx = Some(i);
            //asr::print_message(&format!("FOUND GOAL SYSTEM {}", i));
        }
    }

    if let Some(idx) = goal_system_idx {
        match process.read_pointer_path::<Address64>(
            module.g_world(), 
            asr::PointerSize::Bit64, 
            &vec!(
                0x0 as u64, 
                0x180 as u64, // GameInstance
                0x38 as u64, // LocalPlayers
                0x0 as u64, // Index first array index to get the AlcatrazLocalPlayer instance (maybe make this safer?)
                0xe8 as u64, // Subsystems I think?
                (idx * 24) as u64 + 8 as u64, // index into array to get LocalPlayerGoalSystem
                0x30 as u64, // finally, GoalSystem
            ) 
        ) {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
        }
    } else {
        Ok(asr::Address64::new(0))
    }

    //Ok(asr::Address64::new(0))
}

struct State {
    level_name: String,
    goal_count: u32,
    skater: FNameKey,
    gamemode: u8,
    is_running: bool,
    is_loading: bool,
}

// THPS3+4 doesn't store its goals like the other games: it stores each goal non-linearly (maybe they expected to add more?)
// so we need to construct our own career struct to make it more convenient to both count goals (when AG&G criteria is added) and keep track of medals more easily
struct Career {
    goals: Vec<Vec<bool>>,
}

impl Career {
    pub fn new(process: &Process, base_addr: Address, unreal_module: &asr::game_engine::unreal::Module, skater_fname: FNameKey) -> Self {
        let goals = vec![vec![false; 10]; 17];

        let mut result = Self {
            goals: goals,
        };

        result.update(process, base_addr, unreal_module, skater_fname, 0);

        result
    }

    pub fn reset(&mut self) {
        for i in &mut self.goals {
            for j in 0..i.len() {
                i[j] = false;
            }
        }
    }

    pub fn update(&mut self, process: &Process, goal_system_addr: Address, unreal_module: &asr::game_engine::unreal::Module, skater_fname: FNameKey, old_count: u32) {
        // collect all completed goals and apply them to the career goals
        // go through each career until you find the one for the expected skater
        let career_count = match process.read_pointer_path::<u32>(goal_system_addr, asr::PointerSize::Bit64, &vec!(0xa0 as u64)) {
            Ok(v) => v,
            Err(_) => 0,
        };

        let mut career_offset = -1;
        for i in 0..career_count {
            let career_fname = match process.read_pointer_path::<asr::game_engine::unreal::FNameKey>(goal_system_addr, asr::PointerSize::Bit64, &vec!(0x98 as u64, (i * 0x60) as u64)) {
                Ok(v) => v,
                Err(_) => asr::game_engine::unreal::FNameKey::default(),
            };

            if career_fname == skater_fname {
                career_offset = i as i32;
            }
        }

        if career_offset != -1 {
            let goal_count = match process.read_pointer_path::<u32>(goal_system_addr, asr::PointerSize::Bit64, &vec!(0x98 as u64, (career_offset as u64 * 0x60) + 0x10 as u64)) {
                Ok(v) => v,
                Err(_) => 0,
            };

            for i in old_count..goal_count {
                let goal_fname = match process.read_pointer_path::<asr::game_engine::unreal::FNameKey>(goal_system_addr, asr::PointerSize::Bit64, &vec!(0x98 as u64, (career_offset as u64 * 0x60) + 0x8 as u64, (i as u64 * 0x30) + 0x10 as u64)) {
                    Ok(v) => v,
                    Err(_) => asr::game_engine::unreal::FNameKey::default(),
                };

                let goal_name = get_fname_string(process, unreal_module, goal_fname);

                if let Some((level, idx)) = GOAL_TABLE.get(goal_name.as_str()) {
                    self.goals[*level as usize][*idx as usize] = true;
                } else {
                    asr::print_message(&format!("Unrecognized goal completed: {}", goal_name));
                }
            }
        }
    }
}

impl State {
    pub fn update(process: &Process, goal_system_addr: Address, unreal_module: &asr::game_engine::unreal::Module) -> Self {
        /*let scan_range = 100;
        for i in 0..scan_range {
            match process.read_pointer_path::<FNameKey>(
                goal_system_addr, 
                asr::PointerSize::Bit64, 
                &vec!(
                    0x0 as u64 + (i * 4) as u64,
                )
            ) {
                Ok(key) => {
                    asr::print_message(&format!("OBJ {} ({:#018x}) = {}", i, (i * 4), get_fname_string(process, unreal_module, key)));
                }
                Err(_) => {
                    //asr::print_message(&format!("OBJ {} ({:#018x}) = [INVALID]", i, (i * 8)));
                },
            }
            match process.read_pointer_path::<FNameKey>(
                goal_system_addr, 
                asr::PointerSize::Bit64, 
                &vec!(
                    0x0 as u64,
                    0x180 as u64,
                    0x38 as u64,
                    0x0 as u64,
                    0xe8 as u64,
                    0x98 as u64,
                    0x30 as u64,
                    0x0 as u64 + (i * 8) as u64,
                    0x0 as u64,
                    0x18 as u64,
                )
            ) {
                Ok(key) => {
                    asr::print_message(&format!("OBJ2 {} ({:#018x}) = {}", i, (i * 8), get_fname_string(process, module, key)));
                }
                Err(_) => {},
            }
        }*/

        // name key of skater, we just need to match it to the career, so we don't get the string
        let skater_fname = match process.read_pointer_path::<FNameKey>(goal_system_addr, asr::PointerSize::Bit64, &vec!(0x188 as u64)) {
            Ok(v) => v,
            Err(_) => FNameKey::default(),
        };

        // lol this is very stupid, but I can't find any other way to do it!
        // go through each career until you find the one for the expected skater
        let career_count = match process.read_pointer_path::<u32>(goal_system_addr, asr::PointerSize::Bit64, &vec!(0xa0 as u64)) {
            Ok(v) => v,
            Err(_) => 0,
        };

        let mut career_offset = -1;
        for i in 0..career_count {
            let career_fname = match process.read_pointer_path::<FNameKey>(goal_system_addr, asr::PointerSize::Bit64, &vec!(0x98 as u64, (i * 0x60) as u64)) {
                Ok(v) => v,
                Err(_) => FNameKey::default(),
            };

            if career_fname == skater_fname {
                career_offset = i as i32;
            }
        }

        let mut goal_count = 0;
        if career_offset != -1 {
            goal_count = match process.read_pointer_path::<u32>(goal_system_addr, asr::PointerSize::Bit64, &vec!(0x98 as u64, (career_offset as u64 * 0x60) + 0x10 as u64)) {
                Ok(v) => v,
                Err(_) => 0,
            };
        }

        let level_fname = match process.read_pointer_path::<FNameKey>(asr::Address64::new(0), asr::PointerSize::Bit64, &vec!(unreal_module.g_world().value() as u64, 0x18 as u64)) {
            Ok(v) => v,
            Err(_) => FNameKey::default(),
        };

        let gamemode = match process.read_pointer_path::<u8>(asr::Address64::new(0), asr::PointerSize::Bit64, &vec!(unreal_module.g_world().value() as u64, 0x120 as u64, 0x2D0 as u64)) {
            Ok(v) => v,
            Err(_) => 0,
        };

        let is_running = match process.read_pointer_path::<u8>(asr::Address64::new(0), asr::PointerSize::Bit64, &vec!(unreal_module.g_world().value() as u64, 0x120 as u64, 0x2D2 as u64)) {
            Ok(v) => {
                if v & 0x04 == 0 {
                    // timer has not expired
                    v & 0x01 != 0
                } else {
                    // timer has expired
                    v & 0x01 == 0
                }
            },
            Err(_) => false,
        };

        State {
            level_name: get_fname_string(process, unreal_module, level_fname),

            goal_count: goal_count,

            skater: skater_fname, 

            gamemode,
            is_running,

            is_loading: match process.read_pointer_path::<u8>(asr::Address64::new(0), asr::PointerSize::Bit64, &vec!(unreal_module.g_world().value() as u64, 0x10B as u64)) {
                Ok(v) => v & 0x02 == 0,
                Err(_) => true,
            },
        }
    }
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THPS3+4!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();

    let unreal_module;
    loop {
        asr::print_message("Finding offsets...");
        let offsets_result = asr::game_engine::unreal::Module::attach(process, asr::game_engine::unreal::Version::V4_27, base_addr);
        if let Some(offsets) = offsets_result {
            asr::print_message("Offsets found!");

            asr::print_message(&format!("UENGINE ADDR: {:#018x}", offsets.g_engine().value() - base_addr.value()));
            asr::print_message(&format!("UWORLD ADDR: {:#018x}", offsets.g_world().value() - base_addr.value()));

            unreal_module = offsets;
            break;
        } 
        
        asr::print_message("Failed to find offsets! Trying again...");
        asr::future::next_tick().await;
    }

    asr::print_message(&format!("GENGINE: {:#018x}", unreal_module.g_engine().value() - base_addr.value()));

    // this should stay static once it's created, but we need to make sure we get it, and the world may not exist when the game is started (or if we somehow catch a loading screen)
    let goal_system_addr;
    loop {
        if let Ok(addr) = get_goal_system_pointer(process, &unreal_module) {
            goal_system_addr = asr::Address::new(addr.value());
            break;
        }

        asr::print_message("Failed to get GoalSystem! Trying again...");
        asr::future::next_tick().await;
    }

    asr::print_message(&format!("GOAL SYSTEM ADDR: {:#018x}", goal_system_addr.value()));

    let mut prev_state = State::update(process, goal_system_addr, &unreal_module);
    let mut career = Career::new(process, goal_system_addr, &unreal_module, prev_state.skater);

    let mut roswell_medal = false;
    let mut bullring_medal = false;
    let mut starting_game = false;

    loop {
        // update vars
        let mut current_state = State::update(process, goal_system_addr, &unreal_module);

        // if we see an invalid level name, fill in the previous
        if current_state.level_name.is_empty() {
            current_state.level_name = prev_state.level_name.clone();
        }

        // update career
        if current_state.goal_count != prev_state.goal_count {
            asr::print_message(format!("GOAL COUNT CHANGED TO {}", current_state.goal_count).as_str());
            if current_state.goal_count < prev_state.goal_count || current_state.skater != prev_state.skater {
                roswell_medal = false;
                bullring_medal = false;
                starting_game = false;
                career.reset();
                career.update(process, goal_system_addr, &unreal_module, current_state.skater, 0);
            } else {
                career.update(process, goal_system_addr, &unreal_module, current_state.skater, prev_state.goal_count);
            }
        }

        // pause game time when loading, resume when done
        if current_state.is_loading && !prev_state.is_loading {
            asr::timer::pause_game_time();
            asr::print_message(format!("Starting Load...").as_str());
        } else if !current_state.is_loading && prev_state.is_loading {
            asr::timer::resume_game_time();
            asr::print_message(format!("Done Loading").as_str());
        }

        if current_state.is_running != prev_state.is_running {
            asr::print_message(&format!("is_running changed from {} to {}", prev_state.is_running, current_state.is_running));
        }

        if current_state.skater != prev_state.skater {
            asr::print_message(&format!("skater changed from {} to {}", prev_state.is_running, current_state.is_running));
        }

        //asr::print_message(format!("LEVEL = {}", current_state.level_name).as_str());

        if (current_state.level_name == "Warehouse" || current_state.level_name == "Hangar") && prev_state.level_name == "FrontEnd" {
            starting_game = true;
            asr::print_message(format!("Starting a game").as_str());
        }

        match asr::timer::state() {
            TimerState::NotRunning => {
                // start when no goals have been completed and starting a first level
                if starting_game && current_state.goal_count == 0 && current_state.is_running {
                    if current_state.gamemode == 0x02 {
                        asr::timer::start();
                        asr::print_message(format!("Starting timer...").as_str());
                    }
                    starting_game = false;
                }
            },
            TimerState::Paused | TimerState::Running => {
                // split on level changes (except frontend)
                if !starting_game && !current_state.level_name.is_empty() && current_state.level_name != prev_state.level_name && current_state.level_name != "FrontEnd" {
                    asr::timer::split();
                    asr::print_message(format!("Changed level; splitting timer...").as_str());
                }

                // split when second game is started
                if ((roswell_medal && current_state.level_name == "Hangar") || (bullring_medal && current_state.level_name == "Warehouse")) && starting_game && current_state.is_running {
                    if current_state.gamemode == 0x02 {
                        asr::timer::split();
                        asr::print_message(format!("Changed level; splitting timer...").as_str());
                    }
                    starting_game = false;
                }

                // split when roswell medal is collected
                if career.goals[8][0] && !roswell_medal {
                    roswell_medal = true;
                    asr::timer::split();
                    asr::print_message(format!("Got Roswell medal; splitting timer...").as_str());
                }

                // split when bullring medal is collected
                if career.goals[16][0] && !bullring_medal {
                    bullring_medal = true;
                    asr::timer::split();
                    asr::print_message(format!("Got Bullring medal; splitting timer...").as_str());
                }

                // reset when on frontend with 0 pro points
                if current_state.level_name == "FrontEnd" && current_state.goal_count == 0 {
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

pub fn detect_bootstrap(process: &Process, process_name: &str) -> bool {
    let size = match process.get_module_size(process_name) {
        Ok(v) => v,
        Err(_) => 0,
    };

    // if it's less than 1mb, it's almost definitely the bootstrapper
    if size < 1000000 {
        return true;
    } else {
        return false;
    }
}

// constructs a hash table of every goal in the game to translate from a name to a position in our own career table
static GOAL_TABLE: Lazy<HashMap<&str, (u32, u32)>> = Lazy::new(|| {
    let mut table: HashMap<&str, (u32, u32)> = HashMap::new();

    for (k, v) in GOAL_LIST {
        table.insert(k, v);
    }

    table
});

// list of all goals, in the format (name, (level, index))
const GOAL_LIST: [(&str, (u32, u32)); 0] = [
    /*("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_score_high.warehouse_score_high", (0, 0)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_score_pro.warehouse_score_pro", (0, 1)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_score_sick.warehouse_score_sick", (0, 2)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_score_combo.warehouse_score_combo", (0, 3)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_collect_SKATE.warehouse_collect_SKATE", (0, 4)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_collect_5_items.warehouse_collect_5_items", (0, 5)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_env_5_boxes.warehouse_env_5_boxes", (0, 6)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_BigRail.warehouse_BigRail", (0, 7)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_ChannelGap.warehouse_ChannelGap", (0, 8)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_collect_secret_tape.warehouse_collect_secret_tape", (0, 9)),

    ("/Game/Environments/THPS1/School/Goals/Data/school_score_high.school_score_high", (1, 0)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_score_pro.school_score_pro", (1, 1)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_score_sick.school_score_sick", (1, 2)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_score_combo.school_score_combo", (1, 3)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_collect_SKATE.school_collect_SKATE", (1, 4)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_collect_5_items.school_collect_5_items", (1, 5)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_grind_tables.school_grind_tables", (1, 6)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_wallride_beells.school_wallride_beells", (1, 7)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_heelflip_kicker.school_heelflip_kicker", (1, 8)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_collect_secret_tape.school_collect_secret_tape", (1, 9)),

    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_score_high.Mall_score_high", (2, 0)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_score_pro.Mall_score_pro", (2, 1)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_score_sick.Mall_score_sick", (2, 2)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_score_combo.Mall_score_combo", (2, 3)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_collect_SKATE.Mall_collect_SKATE", (2, 4)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_collect_5_items.Mall_collect_5_items", (2, 5)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_env_directories.Mall_env_directories", (2, 6)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_gap_Slide_CoffeeGrind.Mall_gap_Slide_CoffeeGrind", (2, 7)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_gap_AirWalk_FlylingLeap.Mall_gap_AirWalk_FlylingLeap", (2, 8)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_collect_secret_tape.Mall_collect_secret_tape", (2, 9)),

    ("/Game/Environments/THPS1/Skate/Goals/Data/skate_medal_bronze.skate_medal_bronze", (3, 0)),
    ("/Game/Environments/THPS1/Skate/Goals/Data/skate_medal_silver.skate_medal_silver", (3, 1)),
    ("/Game/Environments/THPS1/Skate/Goals/Data/skate_medal_gold.skate_medal_gold", (3, 2)),

    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_score_high.Downtown_score_high", (4, 0)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_score_pro.Downtown_score_pro", (4, 1)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_score_sick.Downtown_score_sick", (4, 2)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_score_combo.Downtown_score_combo", (4, 3)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_collect_SKATE.Downtown_collect_SKATE", (4, 4)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_collect_5_items.Downtown_collect_5_items", (4, 5)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_collect_NoSkate.Downtown_collect_NoSkate", (4, 6)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_collect_secret_tape.Downtown_collect_secret_tape", (4, 7)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_roofgap_goal.Downtown_roofgap_goal", (4, 8)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_car_goal.Downtown_car_goal", (4, 9)),

    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_score_high.Downhill_score_high", (5, 0)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_score_pro.Downhill_score_pro", (5, 1)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_score_sick.Downhill_score_sick", (5, 2)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_score_combo.Downhill_score_combo", (5, 3)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_Collect_env.Downhill_Collect_env", (5, 4)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_collect_SKATE.Downhill_collect_SKATE", (5, 5)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Donwhill_Collect_5_Items.Donwhill_Collect_5_Items", (5, 6)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_collect_secret_tape.Downhill_collect_secret_tape", (5, 7)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Donwhill_Gap_HazardGap.Donwhill_Gap_HazardGap", (5, 8)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Donwhill_Gaps_Hydrophobic.Donwhill_Gaps_Hydrophobic", (5, 9)),

    ("/Game/Environments/THPS1/Burnside/Goals/Data/burnside_medal_bronze.burnside_medal_bronze", (6, 0)),
    ("/Game/Environments/THPS1/Burnside/Goals/Data/burnside_medal_silver.burnside_medal_silver", (6, 1)),
    ("/Game/Environments/THPS1/Burnside/Goals/Data/burnside_medal_gold.burnside_medal_gold", (6, 2)),

    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_score_high.streets_score_high", (7, 0)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_score_pro.streets_score_pro", (7, 1)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_score_sick.streets_score_sick", (7, 2)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_score_combo.streets_score_combo", (7, 3)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_wreck_cars.streets_wreck_cars", (7, 4)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_collect_SKATE.streets_collect_SKATE", (7, 5)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_collect_5_items.streets_collect_5_items", (7, 6)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_gap_hubba.streets_gap_hubba", (7, 7)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_gap_fountain.streets_gap_fountain", (7, 8)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_collect_secret_tape.streets_collect_secret_tape", (7, 9)),

    ("/Game/Environments/THPS1/Roswell/Goals/Data/Roswell_Medal_Bronze.roswell_medal_bronze", (8, 0)),
    ("/Game/Environments/THPS1/Roswell/Goals/Data/roswell_medal_silver.roswell_medal_silver", (8, 1)),
    ("/Game/Environments/THPS1/Roswell/Goals/Data/roswell_medal_gold.roswell_medal_gold", (8, 2)),

    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_score_high.hanger_score_high", (9, 0)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_score_pro.hanger_score_pro", (9, 1)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_score_sick.hanger_score_sick", (9, 2)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_score_combo.hanger_score_combo", (9, 3)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_collect_SKATE.hanger_collect_SKATE", (9, 4)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_collect_5_items.hanger_collect_5_items", (9, 5)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_env_barrels.hanger_env_barrels", (9, 6)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_gap_hangtime.hanger_gap_hangtime", (9, 7)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_gap_nosegrind.hanger_gap_nosegrind", (9, 8)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_collect_secret_tape.hanger_collect_secret_tape", (9, 9)),

    ("/Game/Environments/THPS2/School2/Goals/Data/school2_score_high.school2_score_high", (10, 0)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_score_pro.school2_score_pro", (10, 1)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_score_sick.school2_score_sick", (10, 2)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_score_combo.school2_score_combo", (10, 3)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_collect_SKATE.school2_collect_SKATE", (10, 4)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_collect_5_items.school2_collect_5_items", (10, 5)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_gap_rails.school2_gap_rails", (10, 6)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_gap_kickflip.school2_gap_kickflip", (10, 7)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_collect_secret_tape.school2_collect_secret_tape", (10, 8)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_wallride_bells.school2_wallride_bells", (10, 9)),

    ("/Game/Environments/THPS2/Marseille/Goals/Gaps/marseille_medal_bronze.marseille_medal_bronze", (11, 0)),
    ("/Game/Environments/THPS2/Marseille/Goals/Gaps/marseille_medal_silver.marseille_medal_silver", (11, 1)),
    ("/Game/Environments/THPS2/Marseille/Goals/Gaps/marseille_medal_gold.marseille_medal_gold", (11, 2)),

    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_score_high.nyc_score_high", (12, 0)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_score_pro.nyc_score_pro", (12, 1)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_score_sick.nyc_score_sick", (12, 2)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_score_combo.nyc_score_combo", (12, 3)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_collect_SKATE.nyc_collect_SKATE", (12, 4)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_collect_5_items.nyc_collect_5_items", (12, 5)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_env_hydrants.nyc_env_hydrants", (12, 6)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_gap_grindrails.nyc_gap_grindrails", (12, 7)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_gap_joeys.nyc_gap_joeys", (12, 8)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_collect_secret_tape.nyc_collect_secret_tape", (12, 9)),

    ("/Game/Environments/THPS2/Venice/Goals/data/venice_score_high.venice_score_high", (13, 0)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_score_pro.venice_score_pro", (13, 1)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_score_sick.venice_score_sick", (13, 2)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_score_combo.venice_score_combo", (13, 3)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_collect_SKATE.venice_collect_SKATE", (13, 4)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_collect_5_items.venice_collect_5_items", (13, 5)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_env_bum.venice_env_bum", (13, 6)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_gap_vb.venice_gap_vb", (13, 7)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_gap_tailslide.venice_gap_tailslide", (13, 8)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_collect_secret_tape.venice_collect_secret_tape", (13, 9)),

    ("/Game/Environments/THPS2/Street/Goals/Data/street_medal_bronze.street_medal_bronze", (14, 0)),
    ("/Game/Environments/THPS2/Street/Goals/Data/street_medal_silver.street_medal_silver", (14, 1)),
    ("/Game/Environments/THPS2/Street/Goals/Data/street_medal_gold.street_medal_gold", (14, 2)),

    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_score_high.philly_score_high", (15, 0)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_score_pro.philly_score_pro", (15, 1)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_score_sick.philly_score_sick", (15, 2)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_score_combo.philly_score_combo", (15, 3)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_collect_SKATE.philly_collect_SKATE", (15, 4)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_collect_5_items.philly_collect_5_items", (15, 5)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_env_valves.philly_env_valves", (15, 6)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_goal_bluntside.philly_goal_bluntside", (15, 7)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_goal_liptrick.philly_goal_liptrick", (15, 8)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_collect_secret_tape.philly_collect_secret_tape", (15, 9)),

    ("/Game/Environments/THPS2/Bullring/Goals/Data/bullring_medal_bronze.bullring_medal_bronze", (16, 0)),
    ("/Game/Environments/THPS2/Bullring/Goals/Data/bullring_medal_silver.bullring_medal_silver", (16, 1)),
    ("/Game/Environments/THPS2/Bullring/Goals/Data/bullring_medal_gold.bullring_medal_gold", (16, 2)),*/
];