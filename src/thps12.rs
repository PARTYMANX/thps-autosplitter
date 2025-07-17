use asr::{timer::TimerState, Process};

use crate::alcatraz_utils;

struct State {
    level_name: String,
    goal_count: u32,
    roswell_medal: bool,
    bullring_medal: bool,
    gamemode: u8,
    is_running: bool,
    is_loading: bool,
}

impl State {
    pub fn update(process: &Process, context: &alcatraz_utils::AlcatrazContext, career: &mut alcatraz_utils::CareerState) -> Self {
        career.update(process, context);

        Self {
            level_name: context.get_level_name(process),
            goal_count: career.get_goal_count(),
            roswell_medal: career.get_goal_state(0, 8, 0),
            bullring_medal: career.get_goal_state(1, 7, 0),
            gamemode: context.get_gamemode(process),
            is_running: context.is_run_active(process),
            is_loading: context.is_loading(process),
        }
    }
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THPS1+2!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();

    let context;
    loop {
        asr::print_message("Finding offsets...");
        let context_result = alcatraz_utils::AlcatrazContext::new(process, base_addr, alcatraz_utils::Game::THPS12);
        if let Some(ctx) = context_result {
            asr::print_message("Offsets found!");

            ctx.list_addresses();

            context = ctx;
            break;
        } 
        
        asr::print_message("Failed to find offsets! Trying again...");
        asr::future::next_tick().await;
    }

    let mut career = context.get_career_state(process);
    let mut prev_state = State::update(process, &context, &mut career);

    let mut starting_game = false;

    loop {
        // update vars
        let mut current_state = State::update(process, &context, &mut career);

        // if we see an invalid level name, fill in the previous
        if current_state.level_name.is_empty() || current_state.level_name == "None" {
            current_state.level_name = prev_state.level_name.clone();
        }

        // pause game time when loading, resume when done
        if current_state.is_loading && !prev_state.is_loading {
            asr::timer::pause_game_time();
            asr::print_message(format!("Starting Load...").as_str());
        } else if !current_state.is_loading && prev_state.is_loading {
            asr::timer::resume_game_time();
            asr::print_message(format!("Done Loading").as_str());
        }

        if (current_state.level_name == "Warehouse" || current_state.level_name == "Hangar") && prev_state.level_name == "FrontEnd" {
            starting_game = true;
            asr::print_message(format!("Starting a game").as_str());
        }

        if starting_game && (current_state.level_name != "Warehouse" && current_state.level_name != "Hangar") {
            starting_game = false;
            asr::print_message(format!("...or not starting a game").as_str());
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
                if ((current_state.roswell_medal && current_state.level_name == "Hangar") || (current_state.bullring_medal && current_state.level_name == "Warehouse")) && starting_game && current_state.is_running {
                    if current_state.gamemode == 0x02 {
                        asr::timer::split();
                        asr::print_message(format!("Changed level; splitting timer...").as_str());
                    }
                    starting_game = false;
                }

                // split when roswell medal is collected
                if current_state.roswell_medal && !prev_state.roswell_medal {
                    asr::timer::split();
                    asr::print_message(format!("Got Roswell medal; splitting timer...").as_str());
                }

                // split when bullring medal is collected
                if current_state.bullring_medal && !prev_state.bullring_medal {
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
