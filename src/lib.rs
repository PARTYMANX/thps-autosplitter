use asr::{Process};

mod thps2;
mod thps3;
mod thps4;

asr::async_main!(stable);

async fn main() {
    loop {
        asr::print_message("Looking for process...");

        let (&name, &game, process) = asr::future::retry(|| {
            PROCESS_NAMES.iter().find_map(|(name, game)| Some((name, game, Process::attach(name)?)))
         }).await;
        process.until_closes(async {
            asr::print_message(format!("Detected {}", name).as_str());

            match game {
                Game::THPS2 => thps2::run(&process, name).await,
                Game::THPS3 => thps3::run(&process, name).await,
                Game::THPS4 => thps4::run(&process, name).await,
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
    // THUG1,
    // THUG2,
    // THAW,
    // THPSHD,
    // THPS12,
}

const PROCESS_NAMES: [(&str, Game); 6] = [
    ("THawk2.exe", Game::THPS2),
    ("THPS2.exe", Game::THPS2),
    ("Skate3.exe", Game::THPS3),
    ("THPS3.exe", Game::THPS3),
    ("Skate4.exe", Game::THPS4),
    ("THPS4.exe", Game::THPS4),
];