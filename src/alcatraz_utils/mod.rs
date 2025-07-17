// utilities for the unreal 4 THPS games (1+2, 3+4)

use asr::game_engine::unreal as asr_unreal;
use goal_table::GOAL_TABLE;

mod goal_table;

pub struct AlcatrazContext {
    unreal_module: asr_unreal::Module,
    _game: Game,
    offsets: Offsets,
}

impl AlcatrazContext {
    pub fn new(process: &asr::Process, main_module_address: asr::Address, game: Game) -> Option<Self> {
        let unreal_version = match game {
            Game::THPS12 => asr_unreal::Version::V4_24,
            Game::THPS34 => asr_unreal::Version::V4_27,
        };

        let unreal_module = match asr_unreal::Module::attach(process, unreal_version, main_module_address) {
            Some(v) => v,
            None => return None,
        };

        let offsets = Offsets::new(process, &unreal_module, game)?;

        Some(Self {
            _game: game,
            unreal_module,
            offsets,
        })
    }

    pub fn get_career_state(&self, process: &asr::Process) -> CareerState {
        CareerState::new(process, self)
    }

    pub fn is_loading(&self, process: &asr::Process) -> bool {
        match process.read_pointer_path::<u8>(self.unreal_module.g_world(), asr::PointerSize::Bit64, &vec![0x0 as u64, self.offsets.loading]) {
            Ok(v) => v & 0x02 == 0,
            Err(_) => true,
        }
    }

    pub fn get_gamemode(&self, process: &asr::Process) -> u8 {
        match process.read_pointer_path::<u8>(self.unreal_module.g_world(), asr::PointerSize::Bit64, &vec![0x0, self.offsets.game_state, self.offsets.subgame_state]) {
            Ok(v) => v,
            Err(_) => 0,
        }
    }

    pub fn is_run_active(&self, process: &asr::Process) -> bool {
        match process.read_pointer_path::<u8>(self.unreal_module.g_world(), asr::PointerSize::Bit64, &vec![0x0 as u64, self.offsets.game_state, self.offsets.subgame_state + 0x2]) {
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
        }
    }

    pub fn get_level_name(&self, process: &asr::Process) -> String {
        let uworld = match self.unreal_module.get_g_world_uobject(process){
            Some(v) => v,
            None => return "".to_string(),
        };
        
        match uworld.get_fname::<128>(process, &self.unreal_module) {
            Ok(cstr) => match cstr.validate_utf8() {
                Ok(str) => str.to_string(),
                Err(_) => "".to_string(),
            },
            Err(_) => "".to_string(),
        }
    }

    fn get_skater_fname(&self, process: &asr::Process) -> asr_unreal::FNameKey {
        match process.read_pointer_path::<asr_unreal::FNameKey>(self.offsets.goal_system.get_address(), asr::PointerSize::Bit64, &vec!(self.offsets.skater_name)) {
            Ok(v) => v,
            Err(_) => asr_unreal::FNameKey::default(),
        }
    }

    pub fn list_addresses(&self) {
        asr::print_message(&format!(""));

        asr::print_message(&format!("GENGINE: {:#018x}", self.unreal_module.g_engine().value()));
        asr::print_message(&format!("GWORLD: {:#018x}", self.unreal_module.g_world().value()));
        self.offsets.list_offsets();

        asr::print_message(&format!(""));
    }
}

fn get_fname_string(process: &asr::Process, module: &asr_unreal::Module, key: asr_unreal::FNameKey) -> String {
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

#[derive(Clone, Copy)]
pub enum Game {
    THPS12,
    THPS34,
}

struct Offsets {
    loading: u64,   // offset from UWorld
    game_state: u64,    // offset from UWorld
    subgame_state: u64, // offset from GameState
    goal_system: asr_unreal::UObject,
    skater_name: u64,   // offset from GoalSystem
    career_count: u64,  // offset from GoalSystem
    careers: u64,   // offset from GoalSystem
}

impl Offsets {
    fn new(process: &asr::Process, unreal_module: &asr_unreal::Module, game: Game) -> Option<Self> {
        let loading = match game {
            Game::THPS12 => 0x11B,
            Game::THPS34 => 0x10B,
        };

        let uworld = unreal_module.get_g_world_uobject(process)?;

        // get GameState
        //let game_state_obj = get_uobject_field(process, unreal_module, &uworld, "GameState")?;
        //game_state_obj.list_fields(process, unreal_module);

        let game_state = uworld.get_field_offset(process, unreal_module, "GameState")? as u64;
        let game_state_obj = match process.read_pointer_path::<asr::Address64>(uworld.get_address(), asr::PointerSize::Bit64, &vec![game_state]) {
            Ok(v) => {
                asr_unreal::UObject::new(asr::Address::new(v.value()))
            },
            Err(_) => return None,
        };

        //game_state_obj.list_fields(process, unreal_module);

        let subgame_state = game_state_obj.get_field_offset(process, unreal_module, "SubGameStateRepInfo")? as u64;

        let goal_system = Self::get_goal_system_pointer(process, unreal_module)?;

        let skater_name = match game {
            Game::THPS12 => 0x130,
            Game::THPS34 => 0x188,
        };

        let career_count = match game {
            Game::THPS12 => 0xe8,   
            Game::THPS34 => 0xf0,   // i'd previously written a0 here, but I have a feeling that was wrong and it really should be e8
        };

        let careers = match game {
            Game::THPS12 => 0xe0,
            Game::THPS34 => 0xe8,   // i'd previously written e8 here, but I have a feeling that was wrong and it really should be f0
        };

        Some(Self {
            loading,
            game_state,
            subgame_state,
            goal_system,
            skater_name,
            career_count,
            careers,
        })
    }

    fn list_offsets(&self) {
        asr::print_message(&format!("OFFSETS:"));
        asr::print_message(&format!("LOADING: {:#018x}", self.loading));
        asr::print_message(&format!("GAME STATE: {:#018x}", self.game_state));
        asr::print_message(&format!("SUBGAME STATE: {:#018x}", self.subgame_state));
        asr::print_message(&format!("GOAL SYSTEM: {:#018x}", self.goal_system.get_address().value()));
        asr::print_message(&format!("SKATER NAME: {:#018x}", self.skater_name));
        asr::print_message(&format!("CAREER COUNT: {:#018x}", self.career_count));
        asr::print_message(&format!("CAREERS: {:#018x}", self.careers));
    }

    fn get_goal_system_pointer(process: &asr::Process, module: &asr_unreal::Module) -> Option<asr_unreal::UObject> {
        let uworld = module.get_g_world_uobject(process)?;
        //uworld.list_fields(process, module);

        // get OwningGameInstance
        let owning_game_instance = Self::get_uobject_field(process, module, &uworld, "OwningGameInstance")?;
        //owning_game_instance.list_fields(process, module);

        // get first index of LocalPlayers
        let local_player_offset = owning_game_instance.get_field_offset(process, module, "LocalPlayers")?;
        let local_player_addr = match process.read_pointer_path::<asr::Address64>(
            owning_game_instance.get_address(), 
            asr::PointerSize::Bit64, 
            &vec!(
                local_player_offset as u64, // LocalPlayers
                0x0 as u64, // index into first object
            ) 
        ) {
            Ok(v) => asr::Address::new(v.value()),
            Err(_) => return None,
        };

        let local_player = asr_unreal::UObject::new(local_player_addr);
        //local_player.list_fields(process, module);

        // find LocalPlayerGoalSystem
        let subsystem_count = match process.read_pointer_path::<u32>(
            local_player.get_address(), 
            asr::PointerSize::Bit64, 
            &vec!(
                0xf0 as u64, // Subsystems I think?
            ) 
        ) {
            Ok(v) => v,
            Err(_) => return None,
        };

        //asr::print_message(&format!("Subsystem count: {}", subsystem_count));

        let mut local_player_goal_system = None;

        for i in 0..subsystem_count {
            match process.read_pointer_path::<asr::Address64>(
                local_player.get_address(), 
                asr::PointerSize::Bit64, 
                &vec!(
                    0xe8 as u64, // Subsystems I think?
                    (i * 24) as u64 + 8 as u64, // index into array
                ) 
            ) {
                Ok(v) => {
                    let object = asr_unreal::UObject::new(asr::Address::new(v.value()));

                    let name = match object.get_fname::<128>(process, module) {
                        Ok(v) => {
                            match v.validate_utf8() {
                                Ok(v) => v.to_string(),
                                Err(_) => "".to_string(),
                            }
                        },
                        Err(_) => "".to_string(),
                    };

                    //asr::print_message(&format!("    {}: {}", i, name));

                    if name == "LocalPlayerGoalSystem" {
                        local_player_goal_system = Some(object);
                        //asr::print_message(&format!("FOUND GOAL SYSTEM {}", i));
                    }
                },
                Err(_) => {},
            }
        }

        if let Some(object) = local_player_goal_system {
            Self::get_uobject_field(process, module, &object, "GoalSystem")
        } else {
            None
        }
    }

    fn get_uobject_field(process: &asr::Process, module: &asr_unreal::Module, object: &asr_unreal::UObject, name: &str) -> Option<asr_unreal::UObject> {
        match object.get_field_offset(process, module, name) {
            Some(offset) => match process.read_pointer_path::<asr::Address64>(object.get_address(), asr::PointerSize::Bit64, &vec![offset as u64]) {
                Ok(addr) => Some(asr_unreal::UObject::new(asr::Address::new(addr.value()))),
                Err(_) => None,
            },
            None => None,
        }
    }
}

// THPS1+2/3+4 doesn't store its goals like the other games: it stores each goal non-linearly (maybe they expected to add more?)
// so we need to construct our own career struct to make it more convenient to both count goals (when AG&G criteria is added) and keep track of medals more easily
pub struct CareerState {
    goals: Vec<Vec<Vec<bool>>>,
    goal_count: u32,
    tours: Vec<TourState>,
    skater: asr_unreal::FNameKey,
}

impl CareerState {
    fn new(process: &asr::Process, context: &AlcatrazContext) -> Self {
        let goals = vec![vec![vec![false; 15]; 10]; 4];
        let tours = vec![TourState::default(); 4];

        let mut result = Self {
            goals,
            goal_count: 0,
            tours, 
            skater: asr_unreal::FNameKey::default()
        };

        result.update(process, context);

        result
    }

    fn reset(&mut self) {
        for tour in &mut self.goals {
            for level in tour {
                level.fill(false);
            }
        }

        self.tours.fill(TourState::default());

        self.goal_count = 0;
    }

    pub fn update(&mut self, process: &asr::Process, context: &AlcatrazContext) {
        let skater_fname = context.get_skater_fname(process);

        if skater_fname != self.skater {
            asr::print_message(&format!("Skater changed, resetting goals"));

            self.reset();

            self.skater = skater_fname;
        }

        // collect all completed goals and apply them to the career goals
        // go through each career until you find the one for the expected skater
        let career_count = match process.read_pointer_path::<u32>(context.offsets.goal_system.get_address(), asr::PointerSize::Bit64, &vec![context.offsets.career_count as u64]) {
            Ok(v) => v,
            Err(_) => 0,
        };

        let mut career_offset = -1;
        for i in 0..career_count {
            let career_fname = match process.read_pointer_path::<asr_unreal::FNameKey>(context.offsets.goal_system.get_address(), asr::PointerSize::Bit64, &vec![context.offsets.careers, (i * 0x60) as u64]) {
                Ok(v) => v,
                Err(_) => asr_unreal::FNameKey::default(),
            };

            if career_fname == skater_fname {
                career_offset = i as i32;
            }
        }

        if career_offset != -1 {
            let goal_count = match process.read_pointer_path::<u32>(context.offsets.goal_system.get_address(), asr::PointerSize::Bit64, &vec!(context.offsets.careers, (career_offset as u64 * 0x60) + 0x10 as u64)) {
                Ok(v) => v,
                Err(_) => 0,
            };

            if goal_count < self.goal_count {
                asr::print_message(&format!("Goal count lower than previous, resetting goals"));
                self.reset();
            }

            let mut has_invalid_goal = false;

            for i in self.goal_count..goal_count {
                let goal_fname = match process.read_pointer_path::<asr_unreal::FNameKey>(context.offsets.goal_system.get_address(), asr::PointerSize::Bit64, &vec!(context.offsets.careers, (career_offset as u64 * 0x60) + 0x8 as u64, (i as u64 * 0x30) + 0x10 as u64)) {
                    Ok(v) => v,
                    Err(_) => asr_unreal::FNameKey::default(),
                };

                let goal_name = get_fname_string(process, &context.unreal_module, goal_fname);

                if let Some((tour, level, idx, ty)) = GOAL_TABLE.get(goal_name.as_str()) {
                    if !self.goals[*tour as usize][*level as usize][*idx as usize] {
                        self.goals[*tour as usize][*level as usize][*idx as usize] = true;

                        match ty {
                            goal_table::GoalType::Normal => {
                                self.tours[*tour as usize].goals += 1;
                            },
                            goal_table::GoalType::Medal => {
                                self.tours[*tour as usize].medals += 1;
                            },
                            goal_table::GoalType::GoldMedal => {
                                self.tours[*tour as usize].medals += 1;
                                self.tours[*tour as usize].gold_medals += 1;
                            },
                            goal_table::GoalType::Pro => {
                                self.tours[*tour as usize].pro_goals += 1;
                            },
                        }
                        //asr::print_message(&format!("Goal completed: {}", goal_name));
                    } else {
                        //asr::print_message(&format!("DUPLICATE GOAL COMPLETED: {}", goal_name));
                        // bizarre bug: when a SKATE goal is completed in 3+4, it expands the goal array by two then the second entry is replaced with the next goal completed
                        // if we see a duplicate, that means that means we see an invalid goal and should not process it
                        has_invalid_goal = true;
                    }
                } else {
                    asr::print_message(&format!("Unrecognized goal completed: {}", goal_name));
                }
            }

            self.goal_count = goal_count;

            if has_invalid_goal {
                self.goal_count -= 1;
            }
        } else {
            // career not found.  reset goals
            self.reset();
        }
    }

    pub fn get_goal_state(&self, tour: u32, level: u32, goal: u32) -> bool {
        self.goals[tour as usize][level as usize][goal as usize]
    }

    pub fn get_goal_count(&self) -> u32 {
        self.goal_count
    }

    pub fn get_tour_state(&self, tour: u32) -> &TourState {
        &self.tours[tour as usize]
    }
}

#[derive(Clone, Copy, Default)]
pub struct TourState {
    pub goals: u32,
    pub pro_goals: u32,
    pub medals: u32,
    pub gold_medals: u32,
}
