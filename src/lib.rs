use asr::Process;

mod thps2;
mod thps3;
mod thps4;
mod thug1;
mod thug2;
mod thaw;
mod thps12;
mod thps34;
mod mhpb;

mod alcatraz_utils;

asr::async_main!(stable);

async fn main() {
    loop {
        asr::print_message("Looking for process...");

        let (&name, &game, process) = asr::future::retry(|| {
            PROCESS_NAMES.iter().find_map(|(name, game)| Some((name, game, find_process(name, game)?)))
        }).await;

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
                Game::THPS34 => thps34::run(&process, name).await,
                Game::MHPB => mhpb::run(&process, name).await,
            }
            
            asr::future::next_tick().await;
        }).await;
    
        asr::print_message("Game Closed");

        if matches!(asr::timer::state(), asr::timer::TimerState::Running) || matches!(asr::timer::state(), asr::timer::TimerState::Paused) {
            asr::timer::reset();
        }

        asr::future::next_tick().await;
    }
}

fn find_process(name: &str, game: &Game) -> Option<Process> {
    if matches!(game, Game::THPS12) {
        let processes = Process::list_by_name(name)?;

        for pid in processes {
            let process = Process::attach_by_pid(pid);

            match &process {
                Some(p) => {
                    if !thps12::detect_bootstrap(&p, name) {
                        return process;
                    }
                },
                None => {
                    // do nothing
                }
            }
        }
        
        None
    } else {
        Process::attach(name)
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
    THPS34,
    MHPB,
}

const PROCESS_NAMES: [(&str, Game); 18] = [
    ("THawk2.exe", Game::THPS2),
    ("THPS2.exe", Game::THPS2),
    ("Skate3.exe", Game::THPS3),
    ("THPS3.exe", Game::THPS3),
    ("Skate4.exe", Game::THPS4),
    ("THPS4.exe", Game::THPS4),
    ("THUG.exe", Game::THUG1),
    ("THUGONE.exe", Game::THUG1),
    ("THUGPM.exe", Game::THUG1),
    ("THUG2.exe", Game::THUG2),
    ("THUGTWO.exe", Game::THUG2),
    ("THUG2PM.exe", Game::THUG2),
    ("THAW.exe", Game::THAW),
    ("THAWPM.exe", Game::THAW),
    ("THPS12.exe", Game::THPS12),
    ("THPS34.exe", Game::THPS34),
    ("BMX.exe", Game::MHPB),
    ("MHPB.exe", Game::MHPB),
];