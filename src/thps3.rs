use asr::{Address, Process, timer::TimerState};

// NOTES:
// POINTERS TO LEVEL IN INTEGER: 0x4e1e90 -> 0x134 -> 0x14 -> 0x690
// POINTERS TO GOAL FLAGS: 0x4e1e90 -> 0x134 -> 0x14 -> 0x564 (EACH SET OF FLAGS IS 8 BYTES, 9 GOALS OR 3 MEDALS.  FOR MEDALS, BRONZE IS LSB)

struct State {
    goal_count: u32,
    medal_count: u32,
    gold_count: u32,
    level_id: u32,
    is_loading: bool,
    is_timer_running: bool,
    is_paused: bool,

    comp_ranking: u32,
    comp_is_over: bool,
}

const LEVEL_COUNT: u32 = 9;
const LEVEL_IS_COMP: [bool; LEVEL_COUNT as usize] = [
    false,
    false,
    true,
    false,
    false,
    true,
    false,
    true,
    false,
];

impl State {
    fn get_goal_count(process: &Process, base_addr: Address) -> u32 {
        let mut result = 0;

        for i in 0..LEVEL_COUNT {
            if !LEVEL_IS_COMP[i as usize] {
                match process.read_pointer_path32::<u32>(base_addr, &vec!(0x4e1e90 as u32, 0x134 as u32, 0x14 as u32, 0x564 as u32 + (i * 8))) {
                    Ok(v) => result += v.count_ones(),
                    Err(_) => {}    // do nothing, we either lost the process or don't have a career initialized
                }
            }
        }

        return result;
    }

    fn get_medal_count(process: &Process, base_addr: Address) -> (u32, u32) {
        let mut num_medals = 0;
        let mut num_gold = 0;

        for i in 0..LEVEL_COUNT {
            if LEVEL_IS_COMP[i as usize] {
                match process.read_pointer_path32::<u32>(base_addr, &vec!(0x4e1e90 as u32, 0x134 as u32, 0x14 as u32, 0x564 as u32 + (i * 8))) {
                    Ok(v) => {
                        if v != 0 {
                            num_medals += 1;
                        }

                        num_gold += match v {
                            0x04 => 1,
                            _ => 0,
                        };
                    }
                    Err(_) => {}    // do nothing, we either lost the process or don't have a career initialized
                }
            }
        }

        return (num_medals, num_gold);
    }

    pub fn update(process: &Process, base_addr: Address) -> Self {
        let (medal_count, gold_count) = Self::get_medal_count(process, base_addr);

        State {
            goal_count: Self::get_goal_count(process, base_addr),
            medal_count, 
            gold_count,

            level_id: match process.read_pointer_path32::<u32>(base_addr, &vec!(0x4e1e90 as u32, 0x134 as u32, 0x14 as u32, 0x690 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            is_loading: match process.read::<bool>(base_addr + 0x1D0620 as u32) {
                Ok(v) => v,
                Err(_) => false,
            },

            is_timer_running: match process.read::<bool>(base_addr + 0x450BC0 as u32) {
                Ok(v) => v,
                Err(_) => false,
            },

            is_paused: match process.read::<bool>(base_addr + 0x450BC8 as u32) {
                Ok(v) => v,
                Err(_) => false,
            },

            comp_ranking: match process.read_pointer_path32::<u32>(base_addr, &vec!(0x4e1e90 as u32, 0x45c as u32, 0x160 as u32)) {
                Ok(v) => v,
                Err(_) => 0,
            },

            comp_is_over: match process.read_pointer_path32::<bool>(base_addr, &vec!(0x4e1e90 as u32, 0x45c as u32, 0x15c as u32)) {
                Ok(v) => v,
                Err(_) => false,
            },
        }
    }
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THPS3!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();

    let mut foundry_started = false;
    let mut tokyo_started = false;
    let mut tokyo_complete = false;
    let mut all_goals_and_golds_complete = false;
    let mut prev_state = State::update(process, base_addr);

    loop {
        // update vars
        let current_state = State::update(process, base_addr);

        // pause game time when loading, resume when done
        if current_state.is_loading && !prev_state.is_loading {
            asr::timer::pause_game_time();
            asr::print_message(format!("Starting Load...").as_str());
        } else if !current_state.is_loading && prev_state.is_loading {
            asr::timer::resume_game_time();
            asr::print_message(format!("Done Loading").as_str());
        }

        if foundry_started && current_state.level_id != 1 {
            foundry_started = false;
        }

        // comp round and placing may be incorrect when starting tokyo, so store when we see the comp start. essentially this tracks initialization
        if !tokyo_started && current_state.level_id == 8 && !current_state.comp_is_over && current_state.medal_count < 3 {
            asr::print_message(format!("Tokyo started!").as_str());
            tokyo_started = true;
        }

        if tokyo_started && current_state.level_id != 8 {
            asr::print_message(format!("Tokyo un-started!").as_str());
            tokyo_started = false;
        }

        if tokyo_complete && current_state.level_id != 8 && current_state.medal_count < 3 {
            tokyo_complete = false;
        }

        if (current_state.goal_count != prev_state.goal_count || current_state.gold_count != prev_state.gold_count) && current_state.goal_count == 54 && current_state.gold_count == 3 {
            all_goals_and_golds_complete = true;
        }

        if all_goals_and_golds_complete && (current_state.goal_count != 54 || current_state.gold_count != 3) {
            all_goals_and_golds_complete = false;
        }

        match asr::timer::state() {
            TimerState::NotRunning => {
                // can't split on level change, so store that it had changed
                if current_state.level_id == 1 && prev_state.level_id == 0 && current_state.goal_count == 0 {
                    foundry_started = true;
                }

                // when goal cams end, start timer
                if foundry_started && !current_state.is_loading && !current_state.is_paused && current_state.is_timer_running {
                    asr::timer::start();
                    asr::print_message(format!("Starting timer...").as_str());
                }
            },
            TimerState::Paused | TimerState::Running => {
                // split on level changes (except skateshop and tokyo to cruise ship, since that's handled when the comp ends)
                if current_state.level_id != prev_state.level_id && current_state.level_id != 0 && prev_state.level_id != 8 && current_state.level_id != 9 {
                    asr::timer::split();
                    asr::print_message(format!("Changed level; splitting timer...").as_str());
                }

                // any% end (end of medal run on tokyo)
                // NOTE: the check is if we're on tokyo, the competition class is initialized, the competition is over, we're in at least 3rd place. 
                // because this has been inconsistent in the past, the medal check is a backup if that fails.  the medal count is only updated after the ceremony, so it's going to be late for splitting
                if !tokyo_complete && (current_state.level_id == 8 && tokyo_started && current_state.comp_is_over && current_state.comp_ranking <= 3) || current_state.medal_count == 3 {
                    tokyo_complete = true;
                    asr::timer::split();
                    asr::print_message(format!("Finished Tokyo; splitting timer...").as_str());
                }

                // ag&g end (all goals and golds collected and run is ended)
                if all_goals_and_golds_complete && !current_state.is_timer_running {
                    all_goals_and_golds_complete = false;
                    asr::timer::split();
                    asr::print_message(format!("Collected all goals and golds; splitting timer...").as_str());
                }

                // reset when going back to skateshop
                if current_state.level_id == 0 && current_state.goal_count == 0 {
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