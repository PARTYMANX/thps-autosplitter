use std::collections::HashMap;
use once_cell::sync::Lazy;

pub enum GoalType {
    Normal,
    Medal,
    GoldMedal,  // makes tracking golds a bit easier
    Pro,    // encompasses pro goals and competition platinums
}

// constructs a hash table of every goal in the game to translate from a name to a position in our own career table
pub static GOAL_TABLE: Lazy<HashMap<&str, (u32, u32, u32, GoalType)>> = Lazy::new(|| {
    let mut table: HashMap<&str, (u32, u32, u32, GoalType)> = HashMap::new();

    for (k, v) in GOAL_LIST {
        table.insert(k, v);
    }

    table
});

// list of all goals, in the format (name, (tour, level, index)) 
// includes all games because i suspect they might merge them someday
const GOAL_LIST: [(&str, (u32, u32, u32, GoalType)); 128] = [
    // THPS1
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_score_high.warehouse_score_high", (0, 0, 0, GoalType::Normal)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_score_pro.warehouse_score_pro", (0, 0, 1, GoalType::Normal)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_score_sick.warehouse_score_sick", (0, 0, 2, GoalType::Normal)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_score_combo.warehouse_score_combo", (0, 0, 3, GoalType::Normal)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_collect_SKATE.warehouse_collect_SKATE", (0, 0, 4, GoalType::Normal)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_collect_5_items.warehouse_collect_5_items", (0, 0, 5, GoalType::Normal)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_env_5_boxes.warehouse_env_5_boxes", (0, 0, 6, GoalType::Normal)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_BigRail.warehouse_BigRail", (0, 0, 7, GoalType::Normal)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_ChannelGap.warehouse_ChannelGap", (0, 0, 8, GoalType::Normal)),
    ("/Game/Environments/THPS1/Warehouse/Goals/Data/warehouse_collect_secret_tape.warehouse_collect_secret_tape", (0, 0, 9, GoalType::Normal)),

    ("/Game/Environments/THPS1/School/Goals/Data/school_score_high.school_score_high", (0, 1, 0, GoalType::Normal)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_score_pro.school_score_pro", (0, 1, 1, GoalType::Normal)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_score_sick.school_score_sick", (0, 1, 2, GoalType::Normal)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_score_combo.school_score_combo", (0, 1, 3, GoalType::Normal)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_collect_SKATE.school_collect_SKATE", (0, 1, 4, GoalType::Normal)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_collect_5_items.school_collect_5_items", (0, 1, 5, GoalType::Normal)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_grind_tables.school_grind_tables", (0, 1, 6, GoalType::Normal)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_wallride_beells.school_wallride_beells", (0, 1, 7, GoalType::Normal)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_heelflip_kicker.school_heelflip_kicker", (0, 1, 8, GoalType::Normal)),
    ("/Game/Environments/THPS1/School/Goals/Data/school_collect_secret_tape.school_collect_secret_tape", (0, 1, 9, GoalType::Normal)),

    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_score_high.Mall_score_high", (0, 2, 0, GoalType::Normal)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_score_pro.Mall_score_pro", (0, 2, 1, GoalType::Normal)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_score_sick.Mall_score_sick", (0, 2, 2, GoalType::Normal)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_score_combo.Mall_score_combo", (0, 2, 3, GoalType::Normal)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_collect_SKATE.Mall_collect_SKATE", (0, 2, 4, GoalType::Normal)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_collect_5_items.Mall_collect_5_items", (0, 2, 5, GoalType::Normal)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_env_directories.Mall_env_directories", (0, 2, 6, GoalType::Normal)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_gap_Slide_CoffeeGrind.Mall_gap_Slide_CoffeeGrind", (0, 2, 7, GoalType::Normal)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_gap_AirWalk_FlylingLeap.Mall_gap_AirWalk_FlylingLeap", (0, 2, 8, GoalType::Normal)),
    ("/Game/Environments/THPS1/Mall/Goals/Data/Mall_collect_secret_tape.Mall_collect_secret_tape", (0, 2, 9, GoalType::Normal)),

    ("/Game/Environments/THPS1/Skate/Goals/Data/skate_medal_bronze.skate_medal_bronze", (0, 3, 0, GoalType::Medal)),
    ("/Game/Environments/THPS1/Skate/Goals/Data/skate_medal_silver.skate_medal_silver", (0, 3, 1, GoalType::Medal)),
    ("/Game/Environments/THPS1/Skate/Goals/Data/skate_medal_gold.skate_medal_gold", (0, 3, 2, GoalType::GoldMedal)),

    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_score_high.Downtown_score_high", (0, 4, 0, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_score_pro.Downtown_score_pro", (0, 4, 1, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_score_sick.Downtown_score_sick", (0, 4, 2, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_score_combo.Downtown_score_combo", (0, 4, 3, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_collect_SKATE.Downtown_collect_SKATE", (0, 4, 4, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_collect_5_items.Downtown_collect_5_items", (0, 4, 5, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_collect_NoSkate.Downtown_collect_NoSkate", (0, 4, 6, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_collect_secret_tape.Downtown_collect_secret_tape", (0, 4, 7, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_roofgap_goal.Downtown_roofgap_goal", (0, 4, 8, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downtown/Goals/Data/Downtown_car_goal.Downtown_car_goal", (0, 4, 9, GoalType::Normal)),

    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_score_high.Downhill_score_high", (0, 5, 0, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_score_pro.Downhill_score_pro", (0, 5, 1, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_score_sick.Downhill_score_sick", (0, 5, 2, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_score_combo.Downhill_score_combo", (0, 5, 3, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_Collect_env.Downhill_Collect_env", (0, 5, 4, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_collect_SKATE.Downhill_collect_SKATE", (0, 5, 5, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Donwhill_Collect_5_Items.Donwhill_Collect_5_Items", (0, 5, 6, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Downhill_collect_secret_tape.Downhill_collect_secret_tape", (0, 5, 7, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Donwhill_Gap_HazardGap.Donwhill_Gap_HazardGap", (0, 5, 8, GoalType::Normal)),
    ("/Game/Environments/THPS1/Downhill/Goals/Data/Donwhill_Gaps_Hydrophobic.Donwhill_Gaps_Hydrophobic", (0, 5, 9, GoalType::Normal)),

    ("/Game/Environments/THPS1/Burnside/Goals/Data/burnside_medal_bronze.burnside_medal_bronze", (0, 6, 0, GoalType::Medal)),
    ("/Game/Environments/THPS1/Burnside/Goals/Data/burnside_medal_silver.burnside_medal_silver", (0, 6, 1, GoalType::Medal)),
    ("/Game/Environments/THPS1/Burnside/Goals/Data/burnside_medal_gold.burnside_medal_gold", (0, 6, 2, GoalType::GoldMedal)),

    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_score_high.streets_score_high", (0, 7, 0, GoalType::Normal)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_score_pro.streets_score_pro", (0, 7, 1, GoalType::Normal)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_score_sick.streets_score_sick", (0, 7, 2, GoalType::Normal)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_score_combo.streets_score_combo", (0, 7, 3, GoalType::Normal)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_wreck_cars.streets_wreck_cars", (0, 7, 4, GoalType::Normal)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_collect_SKATE.streets_collect_SKATE", (0, 7, 5, GoalType::Normal)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_collect_5_items.streets_collect_5_items", (0, 7, 6, GoalType::Normal)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_gap_hubba.streets_gap_hubba", (0, 7, 7, GoalType::Normal)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_gap_fountain.streets_gap_fountain", (0, 7, 8, GoalType::Normal)),
    ("/Game/Environments/THPS1/Streets/Goals/Data/streets_collect_secret_tape.streets_collect_secret_tape", (0, 7, 9, GoalType::Normal)),

    ("/Game/Environments/THPS1/Roswell/Goals/Data/Roswell_Medal_Bronze.roswell_medal_bronze", (0, 8, 0, GoalType::Medal)),
    ("/Game/Environments/THPS1/Roswell/Goals/Data/roswell_medal_silver.roswell_medal_silver", (0, 8, 1, GoalType::Medal)),
    ("/Game/Environments/THPS1/Roswell/Goals/Data/roswell_medal_gold.roswell_medal_gold", (0, 8, 2, GoalType::GoldMedal)),

    // THPS2
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_score_high.hanger_score_high", (1, 0, 0, GoalType::Normal)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_score_pro.hanger_score_pro", (1, 0, 1, GoalType::Normal)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_score_sick.hanger_score_sick", (1, 0, 2, GoalType::Normal)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_score_combo.hanger_score_combo", (1, 0, 3, GoalType::Normal)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_collect_SKATE.hanger_collect_SKATE", (1, 0, 4, GoalType::Normal)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_collect_5_items.hanger_collect_5_items", (1, 0, 5, GoalType::Normal)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_env_barrels.hanger_env_barrels", (1, 0, 6, GoalType::Normal)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_gap_hangtime.hanger_gap_hangtime", (1, 0, 7, GoalType::Normal)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_gap_nosegrind.hanger_gap_nosegrind", (1, 0, 8, GoalType::Normal)),
    ("/Game/Environments/THPS2/Hangar/Goals/Data/hanger_collect_secret_tape.hanger_collect_secret_tape", (1, 0, 9, GoalType::Normal)),

    ("/Game/Environments/THPS2/School2/Goals/Data/school2_score_high.school2_score_high", (1, 1, 0, GoalType::Normal)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_score_pro.school2_score_pro", (1, 1, 1, GoalType::Normal)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_score_sick.school2_score_sick", (1, 1, 2, GoalType::Normal)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_score_combo.school2_score_combo", (1, 1, 3, GoalType::Normal)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_collect_SKATE.school2_collect_SKATE", (1, 1, 4, GoalType::Normal)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_collect_5_items.school2_collect_5_items", (1, 1, 5, GoalType::Normal)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_gap_rails.school2_gap_rails", (1, 1, 6, GoalType::Normal)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_gap_kickflip.school2_gap_kickflip", (1, 1, 7, GoalType::Normal)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_collect_secret_tape.school2_collect_secret_tape", (1, 1, 8, GoalType::Normal)),
    ("/Game/Environments/THPS2/School2/Goals/Data/school2_wallride_bells.school2_wallride_bells", (1, 1, 9, GoalType::Normal)),

    ("/Game/Environments/THPS2/Marseille/Goals/Gaps/marseille_medal_bronze.marseille_medal_bronze", (1, 2, 0, GoalType::Medal)),
    ("/Game/Environments/THPS2/Marseille/Goals/Gaps/marseille_medal_silver.marseille_medal_silver", (1, 2, 1, GoalType::Medal)),
    ("/Game/Environments/THPS2/Marseille/Goals/Gaps/marseille_medal_gold.marseille_medal_gold", (1, 2, 2, GoalType::GoldMedal)),

    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_score_high.nyc_score_high", (1, 3, 0, GoalType::Normal)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_score_pro.nyc_score_pro", (1, 3, 1, GoalType::Normal)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_score_sick.nyc_score_sick", (1, 3, 2, GoalType::Normal)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_score_combo.nyc_score_combo", (1, 3, 3, GoalType::Normal)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_collect_SKATE.nyc_collect_SKATE", (1, 3, 4, GoalType::Normal)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_collect_5_items.nyc_collect_5_items", (1, 3, 5, GoalType::Normal)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_env_hydrants.nyc_env_hydrants", (1, 3, 6, GoalType::Normal)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_gap_grindrails.nyc_gap_grindrails", (1, 3, 7, GoalType::Normal)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_gap_joeys.nyc_gap_joeys", (1, 3, 8, GoalType::Normal)),
    ("/Game/Environments/THPS2/NYC/Goals/Data/nyc_collect_secret_tape.nyc_collect_secret_tape", (1, 3, 9, GoalType::Normal)),

    ("/Game/Environments/THPS2/Venice/Goals/data/venice_score_high.venice_score_high", (1, 4, 0, GoalType::Normal)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_score_pro.venice_score_pro", (1, 4, 1, GoalType::Normal)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_score_sick.venice_score_sick", (1, 4, 2, GoalType::Normal)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_score_combo.venice_score_combo", (1, 4, 3, GoalType::Normal)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_collect_SKATE.venice_collect_SKATE", (1, 4, 4, GoalType::Normal)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_collect_5_items.venice_collect_5_items", (1, 4, 5, GoalType::Normal)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_env_bum.venice_env_bum", (1, 4, 6, GoalType::Normal)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_gap_vb.venice_gap_vb", (1, 4, 7, GoalType::Normal)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_gap_tailslide.venice_gap_tailslide", (1, 4, 8, GoalType::Normal)),
    ("/Game/Environments/THPS2/Venice/Goals/data/venice_collect_secret_tape.venice_collect_secret_tape", (1, 4, 9, GoalType::Normal)),

    ("/Game/Environments/THPS2/Street/Goals/Data/street_medal_bronze.street_medal_bronze", (1, 5, 0, GoalType::Medal)),
    ("/Game/Environments/THPS2/Street/Goals/Data/street_medal_silver.street_medal_silver", (1, 5, 1, GoalType::Medal)),
    ("/Game/Environments/THPS2/Street/Goals/Data/street_medal_gold.street_medal_gold", (1, 5, 2, GoalType::GoldMedal)),

    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_score_high.philly_score_high", (1, 6, 0, GoalType::Normal)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_score_pro.philly_score_pro", (1, 6, 1, GoalType::Normal)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_score_sick.philly_score_sick", (1, 6, 2, GoalType::Normal)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_score_combo.philly_score_combo", (1, 6, 3, GoalType::Normal)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_collect_SKATE.philly_collect_SKATE", (1, 6, 4, GoalType::Normal)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_collect_5_items.philly_collect_5_items", (1, 6, 5, GoalType::Normal)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_env_valves.philly_env_valves", (1, 6, 6, GoalType::Normal)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_goal_bluntside.philly_goal_bluntside", (1, 6, 7, GoalType::Normal)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_goal_liptrick.philly_goal_liptrick", (1, 6, 8, GoalType::Normal)),
    ("/Game/Environments/THPS2/Philly/Goals/Data/philly_collect_secret_tape.philly_collect_secret_tape", (1, 6, 9, GoalType::Normal)),

    ("/Game/Environments/THPS2/Bullring/Goals/Data/bullring_medal_bronze.bullring_medal_bronze", (1, 7, 0, GoalType::Medal)),
    ("/Game/Environments/THPS2/Bullring/Goals/Data/bullring_medal_silver.bullring_medal_silver", (1, 7, 1, GoalType::Medal)),
    ("/Game/Environments/THPS2/Bullring/Goals/Data/bullring_medal_gold.bullring_medal_gold", (1, 7, 2, GoalType::GoldMedal)),

    // THPS3

    // THPS4
];