use std::process;

use asr::Process;

mod thps2;
mod thps3;
mod thps4;
mod thug1;
mod thug2;
mod thaw;
mod thps12;

asr::async_main!(stable);

async fn main() {
    loop {
        asr::print_message("Looking for process...");

        let (&name, &game, processes) = asr::future::retry(|| {
            PROCESS_NAMES.iter().find_map(|(name, game)| Some((name, game, Process::list(name)?)))
        }).await;

        asr::print_message("GOT LIST");

        let process_result = if matches!(game, Game::THPS12) {
            asr::print_message("CHECKING LIST");
            asr::print_message(format!("{} PIDS FOUND:", processes.len()).as_str());
            for i in &processes {
                asr::print_message(format!("{i}").as_str());
            }

            // i hate to put this here, but we have to to make sure this is before process.until_closes()...
            let mut process = Process::attach_pid(processes[0]);
            
            for pid in processes {
                process = Process::attach_pid(pid);

                match &process {
                    Some(p) => {
                        if thps12::detect_bootstrap(&p, name) {
                            asr::print_message("Attached to THPS1+2 bootstrapper.  Trying again...");
                        } else {
                            break;
                        }
                    },
                    None => {
                        // do nothing
                        asr::print_message("FAILED TO ATTACH!!");
                    }
                }
            }
            
            process
        } else {
            Process::attach_pid(processes[0])
        };

        asr::print_message("GET PROCESS");

        let process = process_result.unwrap();

        process.until_closes(async {
            asr::print_message(format!("Detected {}", name).as_str());

            match game {
                Game::THPS2 => thps2::run(&process, name).await,
                Game::THPS3 => thps3::run(&process, name).await,
                Game::THPS4 => thps4::run(&process, name).await,
                Game::THUG1 => thug1::run(&process, name).await,
                Game::THUG2 => thug2::run(&process, name).await,
                Game::THAW => thaw::run(&process, name).await,
                Game::THPS12 => thps12::run(&process, name).await,
            }
            
            asr::future::next_tick().await;
        }).await;
    
        asr::print_message("Game Closed");
        asr::future::next_tick().await;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Game {
    // THPS,    // emulated, not sure how viable it is
    THPS2,
    THPS3,
    THPS4,
    THUG1,
    THUG2,
    THAW,
    // THPSHD,
    THPS12,
}

const PROCESS_NAMES: [(&str, Game); 14] = [
    ("THawk2.exe", Game::THPS2),
    ("Skate3.exe", Game::THPS3),
    ("THPS3.exe", Game::THPS3),
    ("Skate4.exe", Game::THPS4),
    ("THPS4.exe", Game::THPS4),
    ("THUG.exe", Game::THUG1),
    ("THUGONE.exe", Game::THUG1),
    ("THAW.exe", Game::THAW),
    ("THAWPM.exe", Game::THAW),
    ("THAWPM_reloaded.exe", Game::THAW),
    ("THAWPM_polish.exe", Game::THAW),
    ("THUG2.exe", Game::THUG2),
    ("THUGTWO.exe", Game::THUG2),
    ("THPS12.exe", Game::THPS12),
];