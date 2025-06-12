use std::collections::HashMap;

use asr::{Address, Process, timer::TimerState, signature::Signature};
use once_cell::sync::Lazy;

struct Offsets {
    fnamepool: u64,
    uworld: u64,
    career: u64,
    run_state: u64,
}

/* 
    SOME NOTES:
    The run state is not consistent.  I think that's a demonware logging thing which is a bad idea to try and use.  instead I should get the gamemode and hopefully get the timer from there.
    Also the career cannot be found.  is that a slight struct change throwing off the search?  did the code itself change?  there has to be a better way of getting at that state.

    The thing I hope to do is start dumping all objects and figure out what is present in a scene.  hopefully that's not too hard (lol)
*/

impl Offsets {
    pub fn get(process: &Process, base_addr: Address, module_size: u64) -> Self {
        let fnamepool_ptr: Address = match Signature::<11>::new("74 09 48 8D 15 ?? ?? ?? ?? EB 16").scan_process_range(process, (base_addr, module_size)) {
            Some(v) => v + 5,
            None => {
                asr::print_message("Failed to get FNamePool address!!");
                Address::from(0 as u64)
            }
        };
        let uworld_ptr: Address = match Signature::<16>::new("0F 2E ?? 74 ?? 48 8B 1D ?? ?? ?? ?? 48 85 DB 74").scan_process_range(process, (base_addr, module_size)) {
            Some(v) => v + 8,
            None => {
                asr::print_message("Failed to get UWorld address!!");
                Address::from(0 as u64)
            }
        };
        let career_ptr: Address = match Signature::<26>::new("75 DF 48 8D 05 ?? ?? ?? ?? 48 89 5C 24 20 33 dB C7 05 ?? ?? ?? ?? FF FF FF FF").scan_process_range(process, (base_addr, module_size)) {
            Some(v) => v + 18,  // (or add 32 (and add D0 to pointer))
            None => {
                asr::print_message("Failed to get career address!!");
                Address::from(0 as u64)
            }
        };
        let run_state_ptr: Address = match Signature::<15>::new("4C 89 44 24 28 48 89 05 ?? ?? ?? ?? 48 8D 0D").scan_process_range(process, (base_addr, module_size)) {
            Some(v) => v + 8,
            None => {
                asr::print_message("Failed to get run state address!!");
                Address::from(0 as u64)
            }
        };

        Offsets {
            fnamepool: match process.read::<i32>(fnamepool_ptr) {
                Ok(v) => (fnamepool_ptr.value() + 0x4 + v as u64) - base_addr.value(),
                Err(_) => 0,
            },
            uworld: match process.read::<i32>(uworld_ptr) {
                Ok(v) => (uworld_ptr.value() + 0x4 + v as u64) - base_addr.value(),
                Err(_) => 0,
            },
            career: match process.read::<i32>(career_ptr) {
                Ok(v) => ((career_ptr.value() + 0x8 + v as u64) - base_addr.value()) + 0xA0,
                Err(_) => 0,
            },
            run_state: match process.read::<i32>(run_state_ptr) {
                Ok(v) => (run_state_ptr.value() + 0x4 + v as u64) - base_addr.value(),
                Err(_) => 0,
            },
        }
    }

    fn is_valid(&self) -> bool {
        self.fnamepool != 0 && self.uworld != 0 && self.career != 0 && self.run_state != 0
    }

    fn print_offsets(&self) {
        if self.fnamepool != 0 {
            asr::print_message(&format!("FNAMEPOOL ADDR: {:#018x}", self.fnamepool));
        }
        if self.uworld != 0 {
            asr::print_message(&format!("UWORLD ADDR: {:#018x}", self.uworld));
        }
        if self.career != 0 {
            asr::print_message(&format!("CAREER ADDR: {:#018x}", self.career));
        }
        if self.run_state != 0 {
            asr::print_message(&format!("RUN STATE ADDR: {:#018x}", self.run_state));
        }
    }
}

struct State {
    level_name: String,
    goal_count: u32,
    skater: u64,
    is_running: bool,
    is_loading: bool,
}

// Translate an Unreal FName object to its string
// heavily inspired by the autosplitter for Stray (https://github.com/Micrologist/LiveSplit.Stray/blob/main/stray.asl)
fn get_fname(process: &Process, base_addr: Address, offsets: &Offsets, id: u64) -> String {
    if id == 0 {
        return "".to_string();
    }

    let key = (id & u32::MAX as u64) as u32;
    let partial = (id >> 32) as u32;
    let chunk_offset = key >> 16;
    let name_offset = key & u16::MAX as u32;

    let name_entry = match process.read_pointer_path::<i16>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.fnamepool as u64 + ((chunk_offset * 0x8) + 0x10) as u64, (name_offset * 0x2) as u64)) {
        Ok(v) => v,
        Err(_) => 0,
    };

    let name_length = name_entry >> 6;

    let mut result_bytes = vec!();
    for i in 0..name_length {
        let c = match process.read_pointer_path::<u8>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.fnamepool as u64 + ((chunk_offset * 0x8) + 0x10) as u64, ((name_offset * 0x2) + 2 + i as u32) as u64)) {
            Ok(v) => v,
            Err(_) => 0,
        };
        result_bytes.push(c);
    }

    let result = match std::str::from_utf8(&result_bytes) {
        Ok(v) => v,
        Err(_) => "",
    }.to_string();

    if partial == 0 {
        return result;
    } else {
        return result + "_" + &partial.to_string();
    }
}

// THPS1+2 doesn't store its goals like the other games: it stores each goal non-linearly (maybe they expected to add more?)
// so we need to construct our own career struct to make it more convenient to both count goals (when AG&G criteria is added) and keep track of medals more easily
struct Career {
    goals: Vec<Vec<bool>>,
}

impl Career {
    pub fn new(process: &Process, base_addr: Address, offsets: &Offsets, skater_fname: u64) -> Self {
        let goals = vec![vec![false; 10]; 17];

        let mut result = Self {
            goals: goals,
        };

        result.update(process, base_addr, offsets, skater_fname, 0);

        result
    }

    pub fn reset(&mut self) {
        for i in &mut self.goals {
            for j in 0..i.len() {
                i[j] = false;
            }
        }
    }

    pub fn update(&mut self, process: &Process, base_addr: Address, offsets: &Offsets, skater_fname: u64, old_count: u32) {
        // collect all completed goals and apply them to the career goals
        // go through each career until you find the one for the expected skater
        let career_count = match process.read_pointer_path::<u32>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.career as u64, 0xe8 as u64, 0x98 as u64, 0x30 as u64, 0xd8 as u64)) {
            Ok(v) => v,
            Err(_) => 0,
        };

        let mut career_offset = -1;
        for i in 0..career_count {
            let career_fname = match process.read_pointer_path::<u64>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.career as u64, 0xe8 as u64, 0x98 as u64, 0x30 as u64, 0xe0 as u64, (i * 0x60) as u64)) {
                Ok(v) => v,
                Err(_) => 0,
            };

            if career_fname == skater_fname {
                career_offset = i as i32;
            }
        }

        if career_offset != -1 {
            let goal_count = match process.read_pointer_path::<u32>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.career as u64, 0xe8 as u64, 0x98 as u64, 0x30 as u64, 0xe0 as u64, (career_offset as u64 * 0x60) + 0x10 as u64)) {
                Ok(v) => v,
                Err(_) => 0,
            };

            for i in old_count..goal_count {
                let goal_fname = match process.read_pointer_path::<u64>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.career as u64, 0xe8 as u64, 0x98 as u64, 0x30 as u64, 0xe0 as u64, (career_offset as u64 * 0x60) + 0x8 as u64, (i as u64 * 0x30) + 0x10 as u64)) {
                    Ok(v) => v,
                    Err(_) => 0,
                };

                let (level, idx) = GOAL_TABLE[get_fname(process, base_addr, offsets, goal_fname).as_str()];

                self.goals[level as usize][idx as usize] = true;
            }
        }
    }
}

impl State {
    pub fn update(process: &Process, base_addr: Address, offsets: &Offsets) -> Self {
        // name ID of skater, but that does not matter, we just need to match it to the career
        //let skater_fname = match process.read_pointer_path64::<u64>(base_addr, &vec!(0x3d78170 as u64, 0xe8 as u64, 0x98 as u64, 0x30 as u64, 0x130 as u64)) {
        let skater_fname = match process.read_pointer_path::<u64>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.career as u64, 0xe8 as u64, 0x98 as u64, 0x30 as u64, 0x130 as u64)) {
            Ok(v) => v,
            Err(_) => 0,
        };

        // lol this is very stupid, but I can't find any other way to do it!
        // go through each career until you find the one for the expected skater
        let career_count = match process.read_pointer_path::<u32>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.career as u64, 0xe8 as u64, 0x98 as u64, 0x30 as u64, 0xd8 as u64)) {
            Ok(v) => v,
            Err(_) => 0,
        };

        let mut career_offset = -1;
        for i in 0..career_count {
            let career_fname = match process.read_pointer_path::<u64>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.career as u64, 0xe8 as u64, 0x98 as u64, 0x30 as u64, 0xe0 as u64, (i * 0x60) as u64)) {
                Ok(v) => v,
                Err(_) => 0,
            };

            if career_fname == skater_fname {
                career_offset = i as i32;
            }
        }

        let mut goal_count = 0;
        if career_offset != -1 {
            goal_count = match process.read_pointer_path::<u32>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.career as u64, 0xe8 as u64, 0x98 as u64, 0x30 as u64, 0xe0 as u64, (career_offset as u64 * 0x60) + 0x10 as u64)) {
                Ok(v) => v,
                Err(_) => 0,
            };
        }

        let level_fname = match process.read_pointer_path::<u64>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.uworld as u64, 0x18 as u64)) {
            Ok(v) => v,
            Err(_) => 0,
        };

        let msg_len = match process.read_pointer_path::<u8>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.run_state as u64 + 0xD1)) {
            Ok(v) => v,
            Err(_) => 0,
        };

        let mut msg_bytes = vec!();
        for i in 0..msg_len {
            let c = match process.read_pointer_path::<u8>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.run_state as u64 + 0xD2 + i as u64)) {
                Ok(v) => v,
                Err(_) => 0,
            };
            msg_bytes.push(c);
        }

        let msg = match std::str::from_utf8(&msg_bytes) {
            Ok(v) => v,
            Err(_) => "",
        }.to_string();

        //asr::print_message(&format!("MESSAGE: {} {}", msg_len, msg));

        State {
            level_name: get_fname(process, base_addr, &offsets, level_fname),

            goal_count: goal_count,

            skater: skater_fname, 

            // check that the last log message was the start message to know if we're running TODO: in the future when verifying AG&G, we'll need to look for match_end
            is_running: msg == "dlog_event_client_match_start",

            is_loading: match process.read_pointer_path::<u8>(base_addr, asr::PointerSize::Bit64, &vec!(offsets.uworld as u64, 0x10B as u64)) {
                Ok(v) => v & 0x02 == 0,
                Err(_) => true,
            },
        }
    }
}

// TODO: figure out NG+, 3-style any% vs all goals logic

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THPS3+4!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();
    let module_size = process.get_module_size(process_name).unwrap();

    // TODO: loop this until all addresses are found
    let mut offsets;

    loop {
        asr::print_message("Finding offsets...");
        offsets = Offsets::get(process, base_addr, module_size);
        offsets.print_offsets();
        if offsets.is_valid() {
            asr::print_message("Offsets found!");
            break;
        } else {
            // TODO: REMOVE
            asr::print_message("Failed to find offsets! continuing...");
            break;
        }
        
        asr::print_message("Failed to find offsets! Trying again...");
        asr::future::next_tick().await;
    }

    let mut prev_state = State::update(process, base_addr, &offsets);
    let mut career = Career::new(process, base_addr, &offsets, prev_state.skater);

    let mut tokyo_medal = false;
    let mut zoo_medal = false;
    let mut starting_game = false;

    loop {
        // update vars
        let mut current_state = State::update(process, base_addr, &offsets);

        // dump variables just to see what still works
        if current_state.is_loading != prev_state.is_loading {
            asr::print_message(&format!("is_loading changed from {} to {}", prev_state.is_loading, current_state.is_loading));
        }

        if current_state.is_running != prev_state.is_running {
            asr::print_message(&format!("is_running changed from {} to {}", prev_state.is_running, current_state.is_running));
        }

        if current_state.level_name != prev_state.level_name {
            asr::print_message(&format!("level_name changed from {} to {}", prev_state.level_name, current_state.level_name));
        }

        // if we see an invalid level name, fill in the previous
        if current_state.level_name.is_empty() {
            current_state.level_name = prev_state.level_name.clone();
        }

        // update career
        if current_state.goal_count != prev_state.goal_count {
            //asr::print_message(format!("GOAL COUNT CHANGED TO {}", current_state.goal_count).as_str());
            if current_state.goal_count < prev_state.goal_count || current_state.skater != prev_state.skater {
                tokyo_medal = false;
                zoo_medal = false;
                starting_game = false;
                career.reset();
                career.update(process, base_addr, &offsets, current_state.skater, 0);
            } else {
                career.update(process, base_addr, &offsets, current_state.skater, prev_state.goal_count);
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

        //asr::print_message(format!("LEVEL = {}", current_state.level_name).as_str());

        if (current_state.level_name == "Foundry" || current_state.level_name == "College") && prev_state.level_name == "FrontEnd" {
            starting_game = true;
            asr::print_message(format!("Starting a game").as_str());
        }

        match asr::timer::state() {
            TimerState::NotRunning => {
                // start when no goals have been completed and starting a first level
                if starting_game && current_state.goal_count == 0 && current_state.is_running {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());
                    starting_game = false;
                }
            },
            TimerState::Paused | TimerState::Running => {
                // split on level changes (except frontend)
                // TODO: don't split on final comp to bonus level transition
                if !starting_game && !current_state.level_name.is_empty() && current_state.level_name != prev_state.level_name && current_state.level_name != "FrontEnd" {
                    asr::timer::split();
                    asr::print_message(format!("Changed level; splitting timer...").as_str());
                }

                // split when second game is started
                if ((tokyo_medal && current_state.level_name == "College") || (zoo_medal && current_state.level_name == "Foundry")) && starting_game && current_state.is_running {
                    asr::timer::split();
                    asr::print_message(format!("Changed level; splitting timer...").as_str());
                    starting_game = false;
                }

                // split when tokyo medal is collected
                if career.goals[8][0] && !tokyo_medal {
                    tokyo_medal = true;
                    asr::timer::split();
                    asr::print_message(format!("Got Tokyo medal; splitting timer...").as_str());
                }

                // split when zoo medal is collected
                if career.goals[16][0] && !zoo_medal {
                    zoo_medal = true;
                    asr::timer::split();
                    asr::print_message(format!("Got Zoo medal; splitting timer...").as_str());
                }

                // TODO: AG&G splits

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
const GOAL_LIST: [(&str, (u32, u32)); 128] = [
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_score_high.warehouse_score_high", (0, 0)),
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
    ("/Game/Environments/THPS2/Bullring/Goals/Data/bullring_medal_gold.bullring_medal_gold", (16, 2)),
];