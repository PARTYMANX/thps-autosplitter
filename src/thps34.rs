use asr::{timer::TimerState, Process};

use crate::alcatraz_utils;

struct State {
    level_name: String,
    goal_count: u32,
    tokyo_medal: bool,
    zoo_medal: bool,
    thps3_agag: bool,
    thps4_agag: bool,
    gamemode: u8,
    is_running: bool,
    is_loading: bool,
}

impl State {
    pub fn update(process: &Process, context: &alcatraz_utils::AlcatrazContext, career: &mut alcatraz_utils::CareerState) -> Self {
        career.update(process, context);

        let thps3state = career.get_tour_state(2);
        let thps4state = career.get_tour_state(3);

        Self {
            level_name: context.get_level_name(process),
            goal_count: career.get_goal_count(),
            tokyo_medal: career.get_goal_state(2, 7, 0),
            zoo_medal: career.get_goal_state(3, 8, 0),
            thps3_agag: thps3state.goals == 60 && thps3state.gold_medals == 3,
            thps4_agag: thps4state.goals == 70 && thps4state.gold_medals == 3,
            gamemode: context.get_gamemode(process),
            is_running: context.is_run_active(process),
            is_loading: context.is_loading(process),
        }
    }
}

pub async fn run(process: &Process, process_name: &str) {
    asr::print_message("Attached to THPS3+4!");
    asr::set_tick_rate(120.0);  // just in case, explicitly set the tick rate to 120

    let base_addr = process.get_module_address(process_name).unwrap();

    let context;
    loop {
        asr::print_message("Finding offsets...");
        let context_result = alcatraz_utils::AlcatrazContext::new(process, base_addr, alcatraz_utils::Game::THPS34);
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
    let mut ignore_next_level = false;
    let mut thps3_complete = false;
    let mut thps4_complete = false;

    loop {
        // update vars
        let mut current_state = State::update(process, &context, &mut career);

        // if we see an invalid level name, fill in the previous
        if current_state.level_name.is_empty() || current_state.level_name == "None" {
            current_state.level_name = prev_state.level_name.clone();
        }

        // update career
        if current_state.goal_count != prev_state.goal_count {
            asr::print_message(format!("GOAL COUNT CHANGED TO {}", current_state.goal_count).as_str());
            if current_state.goal_count < prev_state.goal_count {
                starting_game = false;
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

        //asr::print_message(format!("LEVEL = {}", current_state.level_name).as_str());

        if (current_state.level_name == "Foundry" || current_state.level_name == "College") && prev_state.level_name == "FrontEnd" {
            starting_game = true;
            asr::print_message(format!("Starting a game").as_str());
        }

        if starting_game && (current_state.level_name != "Foundry" && current_state.level_name != "College") {
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
                    thps3_complete = false;
                    thps4_complete = false;
                    ignore_next_level = false;
                }
            },
            TimerState::Paused | TimerState::Running => {
                // split on level changes (except frontend)
                if !starting_game && !current_state.level_name.is_empty() && current_state.level_name != prev_state.level_name && current_state.level_name != "FrontEnd" {
                    if !ignore_next_level {
                        asr::timer::split();
                        asr::print_message(format!("Changed level; splitting timer...").as_str());
                    } else {
                        asr::print_message("Changed level; Ignoring level split!");
                        ignore_next_level = false;
                    }
                }

                // split when second game is started
                if ((current_state.tokyo_medal && current_state.level_name == "College") || (current_state.zoo_medal && current_state.level_name == "Foundry")) && starting_game && current_state.is_running {
                    if current_state.gamemode == 0x02 {
                        asr::timer::split();
                        asr::print_message(format!("Changed level; splitting timer...").as_str());
                    }
                    starting_game = false;
                }

                // split when tokyo medal is collected
                if current_state.tokyo_medal && !prev_state.tokyo_medal {
                    asr::timer::split();
                    asr::print_message(format!("Got Tokyo medal; splitting timer...").as_str());
                    ignore_next_level = true;
                }

                // split when zoo medal is collected
                if current_state.zoo_medal && !prev_state.zoo_medal {
                    asr::timer::split();
                    asr::print_message(format!("Got Zoo medal; splitting timer...").as_str());
                    ignore_next_level = true;
                }

                // split when all thps3 goals and golds are complete
                if current_state.thps3_agag && !prev_state.thps3_agag {
                    thps3_complete = true;
                    asr::print_message(format!("THPS3 all goals and golds complete; ready to split...").as_str());
                }

                if !current_state.is_running && prev_state.is_running && thps3_complete {
                    asr::timer::split();
                    asr::print_message(format!("THPS3 all goals and golds; splitting timer...").as_str());
                    thps3_complete = false;
                }

                if current_state.thps4_agag && !prev_state.thps4_agag {
                    thps4_complete = true;
                    asr::print_message(format!("THPS4 all goals and golds complete; ready to split...").as_str());
                }

                if !current_state.is_running && prev_state.is_running && thps4_complete {
                    asr::timer::split();
                    asr::print_message(format!("THPS4 all goals and golds; splitting timer...").as_str());
                    thps4_complete = false;
                }

                // reset when on frontend with 0 pro points
                if current_state.level_name == "FrontEnd" && current_state.goal_count == 0 {
                    asr::timer::reset();
                    asr::print_message(format!("Resetting timer...").as_str());

                    thps3_complete = false;
                    thps4_complete = false;
                    ignore_next_level = false;
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
