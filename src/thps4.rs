use asr::{Address, Process, timer::TimerState};

use crate::context::SplitterContext;

struct State {
    level_id: u8,
    total_cash: u32,
    pro_points: u8,
    pro_goals_completed: u8,
    is_loading: bool,
}

// TODO: add splits for other categories!!

impl State {
    pub fn update(process: &Process, base_addr: Address) -> Self {
        State {
            // TODO: change this to use level ID
            level_id: match process.read_pointer_path::<u8>(
                base_addr,
                asr::PointerSize::Bit32,
                &vec![0x6B5B48 as u64, 0x20 as u64, 0x484 as u64],
            ) {
                Ok(v) => v,
                Err(_) => 0,
            },

            total_cash: match process.read_pointer_path::<u32>(
                base_addr,
                asr::PointerSize::Bit32,
                &vec![0x6B5B48 as u64, 0x86c as u64, 0x28 as u64],
            ) {
                Ok(v) => v,
                Err(_) => 0,
            },

            pro_points: match process.read_pointer_path::<u8>(
                base_addr,
                asr::PointerSize::Bit32,
                &vec![0x6B5B48 as u64, 0x86c as u64, 0x20 as u64],
            ) {
                Ok(v) => v,
                Err(_) => 0,
            },

            pro_goals_completed: match process.read_pointer_path::<u32>(
                base_addr,
                asr::PointerSize::Bit32,
                &vec![0x6B5B48 as u64, 0x20 as u64, 0x454 as u64],
            ) {
                Ok(v) => v.count_ones() as u8,
                Err(_) => 0,
            },

            is_loading: match process.read::<bool>(base_addr + 0x6728C0 as u32) {
                Ok(v) => v,
                Err(_) => false,
            },
        }
    }
}

pub async fn run(process: &Process, process_name: &str, context: &mut SplitterContext) {
    asr::print_message("Attached to THPS4!");
    asr::set_tick_rate(120.0); // just in case, explicitly set the tick rate to 120

    context.update();

    let base_addr = process.get_module_address(process_name).unwrap();

    let mut prev_state = State::update(process, base_addr);

    loop {
        context.update();

        // update vars
        let current_state = State::update(process, base_addr);

        // pause game time when loading, resume when done
        if current_state.is_loading && !prev_state.is_loading {
            context.timer_pause_game_time();
            asr::print_message(format!("Starting Load...").as_str());
        } else if !current_state.is_loading && prev_state.is_loading {
            context.timer_resume_game_time();
            asr::print_message(format!("Done Loading").as_str());
        }

        match asr::timer::state() {
            TimerState::NotRunning => {
                if current_state.level_id == 1
                    && prev_state.level_id == 0
                    && current_state.pro_points == 0
                {
                    context.timer_start();
                }
            }
            TimerState::Paused | TimerState::Running => {
                // split on level changes (except skateshop)
                if current_state.level_id != prev_state.level_id && current_state.level_id != 0 {
                    context.timer_split("Changed level");
                }

                if current_state.pro_goals_completed > prev_state.pro_goals_completed
                    && prev_state.pro_goals_completed == 0
                {
                    context.timer_split("Completed pro goal");
                }

                // split when all goals cleared (190 pro points)
                if current_state.pro_points != prev_state.pro_points
                    && current_state.pro_points == 190
                {
                    context.timer_split("Completed all goals");
                }

                // split on all cash collected
                if current_state.total_cash != prev_state.total_cash
                    && current_state.total_cash == 100000
                {
                    context.timer_split("All cash collected");
                }

                // reset when on skateshop with 0 pro points
                if current_state.level_id == 0 && current_state.pro_points == 0 {
                    context.timer_reset(false);
                }
            }
            TimerState::Ended | TimerState::Unknown | _ => {
                // reset when on skateshop with 0 pro points
                if current_state.level_id == 0 && current_state.pro_points == 0 {
                    context.timer_reset(true);
                }
            }
        }

        prev_state = current_state;

        asr::future::next_tick().await;
    }
}
