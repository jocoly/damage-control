use rdev::{listen, EventType};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{Manager, WindowEvent};

mod settings;

use settings::{
    apply_app_settings, get_app_settings, load_settings_file, update_app_settings, SettingsState,
    SETTINGS_FILE_NAME,
};

const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(15);
const SAVE_FILE_NAME: &str = "save.json";
const SAVE_VERSION: u32 = 1;
const POWER_PROC_ROLL_INTERVAL_MILLIS: u64 = 1_000;
const ROYAL_CONTRACT_ID: &str = "royal_contract";
const CRUMPLED_COURT_ONBOARDING_MANUAL_ID: &str = "crumpled_court_onboarding_manual";
const CHANCE_SCALE: u64 = 1_000_000;
const SHOP_ITEMS: &[ShopItem] = &[
    ShopItem::new(ROYAL_CONTRACT_ID, 100, 2, ShopItemCategory::OrgChart, None),
    ShopItem::new(
        CRUMPLED_COURT_ONBOARDING_MANUAL_ID,
        100,
        3,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(7_500, 25)),
    ),
    ShopItem::new(
        "bottomless_court_coffee_mug",
        250,
        6,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(6_000, 50)),
    ),
    ShopItem::new(
        "royal_suggestion_box_of_destiny",
        500,
        9,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(5_000, 100)),
    ),
    ShopItem::new(
        "royal_crier_app_subscription",
        900,
        12,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(4_000, 150)),
    ),
    ShopItem::new(
        "ledger_of_courtly_accounting",
        1_500,
        15,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(3_200, 250)),
    ),
    ShopItem::new(
        "crystal_audience_analytics_orb",
        2_500,
        18,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(2_600, 400)),
    ),
    ShopItem::new(
        "enchanted_press_release_quill",
        4_000,
        21,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(2_200, 600)),
    ),
    ShopItem::new(
        "royal_mimic_stamp",
        6_000,
        24,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(1_800, 900)),
    ),
    ShopItem::new(
        "royal_messaging_handbook_revised_edition",
        8_500,
        27,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(1_500, 1_200)),
    ),
    ShopItem::new(
        "court_newsletter_press",
        12_000,
        30,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(1_200, 1_800)),
    ),
    ShopItem::new(
        "goblin_outreach_playbook",
        17_000,
        33,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(1_000, 2_500)),
    ),
    ShopItem::new(
        "royal_quest_board",
        23_000,
        36,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(800, 3_500)),
    ),
    ShopItem::new(
        "arcane_audience_survey_scrolls",
        30_000,
        39,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(700, 4_500)),
    ),
    ShopItem::new(
        "court_recruitment_poster_set",
        40_000,
        42,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(600, 6_000)),
    ),
    ShopItem::new(
        "royal_courier_satchel",
        55_000,
        45,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(500, 8_000)),
    ),
    ShopItem::new(
        "lute_of_royal_ballads",
        75_000,
        48,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(400, 12_000)),
    ),
    ShopItem::new(
        "royal_sponsorship_contract",
        100_000,
        51,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(350, 16_000)),
    ),
    ShopItem::new(
        "runic_royal_printing_press",
        130_000,
        54,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(300, 22_000)),
    ),
    ShopItem::new(
        "dragon_egg_aethernet_cluster",
        170_000,
        57,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(260, 30_000)),
    ),
    ShopItem::new(
        "tome_of_royal_memes",
        220_000,
        60,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(220, 40_000)),
    ),
    ShopItem::new(
        "royal_public_relations_handbook",
        280_000,
        63,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(190, 55_000)),
    ),
    ShopItem::new(
        "patent_pending_campaign_spellbook",
        350_000,
        66,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(160, 75_000)),
    ),
    ShopItem::new(
        "arcane_audience_research_journal",
        450_000,
        69,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(140, 95_000)),
    ),
    ShopItem::new(
        "royal_aetherwave_broadcast_tower",
        575_000,
        72,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(120, 125_000)),
    ),
    ShopItem::new(
        "royal_expedition_contract_ledger",
        725_000,
        75,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(100, 170_000)),
    ),
    ShopItem::new(
        "dragon_endorsed_royal_campaign",
        900_000,
        78,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(90, 220_000)),
    ),
    ShopItem::new(
        "royal_census_crystal",
        1_100_000,
        81,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(80, 280_000)),
    ),
    ShopItem::new(
        "royal_prophecy_engine",
        1_350_000,
        84,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(70, 360_000)),
    ),
    ShopItem::new(
        "grand_royal_campaign_blueprint",
        1_650_000,
        87,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(60, 475_000)),
    ),
    ShopItem::new(
        "aethernet_kingdom_news_license",
        2_000_000,
        90,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(50, 650_000)),
    ),
    ShopItem::new(
        "aethernet_data_core",
        2_500_000,
        93,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(40, 900_000)),
    ),
    ShopItem::new(
        "royal_influence_exchange_charter",
        3_100_000,
        96,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(35, 1_200_000)),
    ),
    ShopItem::new(
        "crown_of_the_aethernet_algorithm_dragon",
        4_000_000,
        99,
        ShopItemCategory::PowerUpgrade,
        Some(PowerUpgradeEffect::new(30, 1_750_000)),
    ),
];

struct ShopItem {
    id: &'static str,
    cost: u64,
    required_level: u64,
    category: ShopItemCategory,
    power_upgrade_effect: Option<PowerUpgradeEffect>,
}

impl ShopItem {
    const fn new(
        id: &'static str,
        cost: u64,
        required_level: u64,
        category: ShopItemCategory,
        power_upgrade_effect: Option<PowerUpgradeEffect>,
    ) -> Self {
        Self {
            id,
            cost,
            required_level,
            category,
            power_upgrade_effect,
        }
    }
}

#[derive(Clone, Copy)]
struct PowerUpgradeEffect {
    chance_per_million_inputs: u64,
    reward: u64,
}

impl PowerUpgradeEffect {
    const fn new(chance_per_million_inputs: u64, reward: u64) -> Self {
        Self {
            chance_per_million_inputs,
            reward,
        }
    }
}

#[derive(Clone, Copy)]
enum ShopItemCategory {
    OrgChart,
    PowerUpgrade,
}

#[derive(Default)]
struct InputCounts {
    keys: AtomicU64,
    clicks: AtomicU64,
    bonus_influence: AtomicU64,
    spent_influence: AtomicU64,
    crumpled_court_onboarding_manual_input_baseline: AtomicU64,
    crumpled_court_onboarding_manual_trigger_count: AtomicU64,
    pending_power_proc_inputs: AtomicU64,
    last_power_proc_roll_at_millis: AtomicU64,
    random_state: AtomicU64,
    power_event_sequence: AtomicU64,
    last_power_event_at_millis: AtomicU64,
    last_power_event_amount: AtomicU64,
    last_input_at_millis: AtomicU64,
    last_global_key_at_millis: AtomicU64,
    inventory: Mutex<Inventory>,
    dirty: AtomicBool,
}

#[derive(Serialize)]
struct InputSnapshot {
    influence: u64,
    xp: u64,
    level: u64,
    xp_for_current_level: u64,
    xp_for_next_level: u64,
    keys: u64,
    clicks: u64,
    bonus_influence: u64,
    power_event_sequence: u64,
    last_power_event_at_millis: u64,
    last_power_event_amount: u64,
    inventory_item_ids: Vec<String>,
    last_input_at_millis: u64,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SaveData {
    version: u32,
    keys: u64,
    clicks: u64,
    #[serde(default)]
    bonus_influence: u64,
    #[serde(default)]
    spent_influence: u64,
    #[serde(default)]
    crumpled_court_onboarding_manual_input_baseline: u64,
    #[serde(default)]
    crumpled_court_onboarding_manual_trigger_count: u64,
    #[serde(default)]
    pending_power_proc_inputs: u64,
    #[serde(default)]
    last_power_proc_roll_at_millis: u64,
    #[serde(default)]
    inventory: Inventory,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
struct Inventory {
    #[serde(default)]
    org_chart: Vec<String>,
    #[serde(default)]
    power_upgrades: Vec<String>,
}

impl Inventory {
    fn item_ids(&self) -> Vec<String> {
        self.org_chart
            .iter()
            .chain(self.power_upgrades.iter())
            .cloned()
            .collect()
    }

    fn has_item(&self, item_id: &str) -> bool {
        self.org_chart
            .iter()
            .chain(self.power_upgrades.iter())
            .any(|owned_id| owned_id == item_id)
    }

    fn add_item(&mut self, item: &ShopItem) {
        match item.category {
            ShopItemCategory::OrgChart => self.org_chart.push(item.id.to_string()),
            ShopItemCategory::PowerUpgrade => self.power_upgrades.push(item.id.to_string()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum PurchaseStatus {
    Purchased,
    AlreadyOwned,
    NotEnoughInfluence,
    Locked,
    UnknownItem,
}

#[derive(Serialize)]
struct PurchaseResult {
    status: PurchaseStatus,
    snapshot: InputSnapshot,
}

#[derive(Serialize)]
struct RuntimeConfig {
    presentation_mode: String,
}

impl InputCounts {
    fn snapshot(&self) -> InputSnapshot {
        self.flush_due_power_upgrades(current_time_millis());

        let keys = self.keys.load(Ordering::Relaxed);
        let clicks = self.clicks.load(Ordering::Relaxed);
        let bonus_influence = self.bonus_influence.load(Ordering::Relaxed);
        let spent_influence = self.spent_influence.load(Ordering::Relaxed);
        let xp = keys + clicks + bonus_influence;
        let level = level_for_xp(xp);
        let inventory = self.inventory.lock().expect("inventory lock poisoned");

        InputSnapshot {
            influence: xp.saturating_sub(spent_influence),
            xp,
            level,
            xp_for_current_level: xp_required_for_level(level),
            xp_for_next_level: xp_required_for_level(level + 1),
            keys,
            clicks,
            bonus_influence,
            power_event_sequence: self.power_event_sequence.load(Ordering::Relaxed),
            last_power_event_at_millis: self.last_power_event_at_millis.load(Ordering::Relaxed),
            last_power_event_amount: self.last_power_event_amount.load(Ordering::Relaxed),
            inventory_item_ids: inventory.item_ids(),
            last_input_at_millis: self.last_input_at_millis.load(Ordering::Relaxed),
        }
    }

    fn snapshot_save(&self) -> SaveData {
        self.flush_due_power_upgrades(current_time_millis());

        SaveData {
            version: SAVE_VERSION,
            keys: self.keys.load(Ordering::Relaxed),
            clicks: self.clicks.load(Ordering::Relaxed),
            bonus_influence: self.bonus_influence.load(Ordering::Relaxed),
            spent_influence: self.spent_influence.load(Ordering::Relaxed),
            crumpled_court_onboarding_manual_input_baseline: self
                .crumpled_court_onboarding_manual_input_baseline
                .load(Ordering::Relaxed),
            crumpled_court_onboarding_manual_trigger_count: self
                .crumpled_court_onboarding_manual_trigger_count
                .load(Ordering::Relaxed),
            pending_power_proc_inputs: self.pending_power_proc_inputs.load(Ordering::Relaxed),
            last_power_proc_roll_at_millis: self
                .last_power_proc_roll_at_millis
                .load(Ordering::Relaxed),
            inventory: self
                .inventory
                .lock()
                .expect("inventory lock poisoned")
                .clone(),
        }
    }

    fn load_save(&self, save_data: SaveData) {
        let crumpled_court_onboarding_manual_input_baseline = if save_data
            .inventory
            .has_item(CRUMPLED_COURT_ONBOARDING_MANUAL_ID)
            && save_data.crumpled_court_onboarding_manual_input_baseline == 0
            && save_data.crumpled_court_onboarding_manual_trigger_count == 0
        {
            save_data.keys + save_data.clicks
        } else {
            save_data.crumpled_court_onboarding_manual_input_baseline
        };

        self.keys.store(save_data.keys, Ordering::Relaxed);
        self.clicks.store(save_data.clicks, Ordering::Relaxed);
        self.bonus_influence
            .store(save_data.bonus_influence, Ordering::Relaxed);
        self.spent_influence
            .store(save_data.spent_influence, Ordering::Relaxed);
        self.crumpled_court_onboarding_manual_input_baseline.store(
            crumpled_court_onboarding_manual_input_baseline,
            Ordering::Relaxed,
        );
        self.crumpled_court_onboarding_manual_trigger_count.store(
            save_data.crumpled_court_onboarding_manual_trigger_count,
            Ordering::Relaxed,
        );
        self.pending_power_proc_inputs
            .store(save_data.pending_power_proc_inputs, Ordering::Relaxed);
        self.last_power_proc_roll_at_millis
            .store(save_data.last_power_proc_roll_at_millis, Ordering::Relaxed);
        *self.inventory.lock().expect("inventory lock poisoned") = save_data.inventory;
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn reset_progress(&self) -> InputSnapshot {
        self.keys.store(0, Ordering::Relaxed);
        self.clicks.store(0, Ordering::Relaxed);
        self.bonus_influence.store(0, Ordering::Relaxed);
        self.spent_influence.store(0, Ordering::Relaxed);
        self.crumpled_court_onboarding_manual_input_baseline
            .store(0, Ordering::Relaxed);
        self.crumpled_court_onboarding_manual_trigger_count
            .store(0, Ordering::Relaxed);
        self.pending_power_proc_inputs.store(0, Ordering::Relaxed);
        self.last_power_proc_roll_at_millis
            .store(0, Ordering::Relaxed);
        self.random_state.store(0, Ordering::Relaxed);
        self.power_event_sequence.store(0, Ordering::Relaxed);
        self.last_power_event_at_millis.store(0, Ordering::Relaxed);
        self.last_power_event_amount.store(0, Ordering::Relaxed);
        self.last_input_at_millis.store(0, Ordering::Relaxed);
        self.last_global_key_at_millis.store(0, Ordering::Relaxed);
        *self.inventory.lock().expect("inventory lock poisoned") = Inventory::default();
        self.mark_dirty();

        self.snapshot()
    }

    fn dev_add_influence(&self, amount: u64) -> InputSnapshot {
        self.keys.fetch_add(amount, Ordering::Relaxed);
        let input_at_millis = current_time_millis();
        self.record_input_time_at(input_at_millis);
        self.queue_power_upgrade_inputs(input_at_millis, amount);
        self.mark_dirty();

        self.snapshot()
    }

    fn purchase_shop_item(&self, item_id: &str) -> PurchaseResult {
        self.flush_due_power_upgrades(current_time_millis());

        let Some(item) = find_shop_item(item_id) else {
            return self.purchase_result(PurchaseStatus::UnknownItem);
        };

        if level_for_xp(self.lifetime_xp()) < item.required_level {
            return self.purchase_result(PurchaseStatus::Locked);
        }

        let mut inventory = self.inventory.lock().expect("inventory lock poisoned");

        if item.id != ROYAL_CONTRACT_ID && !inventory.has_item(ROYAL_CONTRACT_ID) {
            return self.purchase_result_with_inventory(PurchaseStatus::Locked, inventory);
        }

        if inventory.has_item(item_id) {
            return self.purchase_result_with_inventory(PurchaseStatus::AlreadyOwned, inventory);
        }

        let available_influence = self.available_influence();

        if available_influence < item.cost {
            return self
                .purchase_result_with_inventory(PurchaseStatus::NotEnoughInfluence, inventory);
        }

        inventory.add_item(item);
        if item.id == CRUMPLED_COURT_ONBOARDING_MANUAL_ID {
            self.crumpled_court_onboarding_manual_input_baseline
                .store(self.raw_input_total(), Ordering::Relaxed);
            self.crumpled_court_onboarding_manual_trigger_count
                .store(0, Ordering::Relaxed);
            self.pending_power_proc_inputs.store(0, Ordering::Relaxed);
            self.last_power_proc_roll_at_millis
                .store(0, Ordering::Relaxed);
        }
        self.spent_influence.fetch_add(item.cost, Ordering::Relaxed);
        self.mark_dirty();

        self.purchase_result_with_inventory(PurchaseStatus::Purchased, inventory)
    }

    fn purchase_result(&self, status: PurchaseStatus) -> PurchaseResult {
        PurchaseResult {
            status,
            snapshot: self.snapshot(),
        }
    }

    fn available_influence(&self) -> u64 {
        let keys = self.keys.load(Ordering::Relaxed);
        let clicks = self.clicks.load(Ordering::Relaxed);
        let bonus_influence = self.bonus_influence.load(Ordering::Relaxed);

        (keys + clicks + bonus_influence)
            .saturating_sub(self.spent_influence.load(Ordering::Relaxed))
    }

    fn lifetime_xp(&self) -> u64 {
        self.keys.load(Ordering::Relaxed)
            + self.clicks.load(Ordering::Relaxed)
            + self.bonus_influence.load(Ordering::Relaxed)
    }

    fn raw_input_total(&self) -> u64 {
        self.keys.load(Ordering::Relaxed) + self.clicks.load(Ordering::Relaxed)
    }

    fn purchase_result_with_inventory(
        &self,
        status: PurchaseStatus,
        inventory: std::sync::MutexGuard<'_, Inventory>,
    ) -> PurchaseResult {
        let keys = self.keys.load(Ordering::Relaxed);
        let clicks = self.clicks.load(Ordering::Relaxed);
        let bonus_influence = self.bonus_influence.load(Ordering::Relaxed);
        let spent_influence = self.spent_influence.load(Ordering::Relaxed);
        let xp = keys + clicks + bonus_influence;
        let level = level_for_xp(xp);

        PurchaseResult {
            status,
            snapshot: InputSnapshot {
                influence: xp.saturating_sub(spent_influence),
                xp,
                level,
                xp_for_current_level: xp_required_for_level(level),
                xp_for_next_level: xp_required_for_level(level + 1),
                keys,
                clicks,
                bonus_influence,
                power_event_sequence: self.power_event_sequence.load(Ordering::Relaxed),
                last_power_event_at_millis: self.last_power_event_at_millis.load(Ordering::Relaxed),
                last_power_event_amount: self.last_power_event_amount.load(Ordering::Relaxed),
                inventory_item_ids: inventory.item_ids(),
                last_input_at_millis: self.last_input_at_millis.load(Ordering::Relaxed),
            },
        }
    }

    fn record_event(&self, event_type: EventType) {
        match event_type {
            EventType::KeyPress(_) => {
                let input_at_millis = current_time_millis();
                self.record_keypress(input_at_millis);
                self.last_global_key_at_millis
                    .store(input_at_millis, Ordering::Relaxed);
            }
            EventType::ButtonPress(_) => {
                let input_at_millis = current_time_millis();
                self.clicks.fetch_add(1, Ordering::Relaxed);
                self.record_input_time_at(input_at_millis);
                self.queue_power_upgrade_inputs(input_at_millis, 1);
                self.mark_dirty();
            }
            _ => {}
        }
    }

    fn record_focused_keypress(&self, event_at_millis: u64) {
        let last_global_key_at_millis = self.last_global_key_at_millis.load(Ordering::Relaxed);

        if event_at_millis.abs_diff(last_global_key_at_millis) <= 100 {
            return;
        }

        self.record_keypress(event_at_millis);
    }

    fn record_keypress(&self, input_at_millis: u64) {
        self.keys.fetch_add(1, Ordering::Relaxed);
        self.record_input_time_at(input_at_millis);
        self.queue_power_upgrade_inputs(input_at_millis, 1);
        self.mark_dirty();
    }

    fn queue_power_upgrade_inputs(&self, input_at_millis: u64, input_count: u64) {
        let has_manual = self
            .inventory
            .lock()
            .expect("inventory lock poisoned")
            .has_item(CRUMPLED_COURT_ONBOARDING_MANUAL_ID);

        if !has_manual {
            return;
        }

        self.pending_power_proc_inputs
            .fetch_add(input_count, Ordering::Relaxed);
        self.last_power_proc_roll_at_millis
            .compare_exchange(0, input_at_millis, Ordering::Relaxed, Ordering::Relaxed)
            .ok();
    }

    fn flush_due_power_upgrades(&self, now_millis: u64) {
        let last_roll_at_millis = self.last_power_proc_roll_at_millis.load(Ordering::Relaxed);

        if last_roll_at_millis == 0
            || now_millis.saturating_sub(last_roll_at_millis) < POWER_PROC_ROLL_INTERVAL_MILLIS
        {
            return;
        }

        let input_count = self.pending_power_proc_inputs.swap(0, Ordering::Relaxed);

        if input_count == 0 {
            self.last_power_proc_roll_at_millis
                .store(now_millis, Ordering::Relaxed);
            return;
        }

        self.last_power_proc_roll_at_millis
            .store(now_millis, Ordering::Relaxed);
        self.process_power_upgrade_rolls(now_millis, input_count);
    }

    fn process_power_upgrade_rolls(&self, input_at_millis: u64, input_count: u64) {
        let owned_power_upgrade_ids = self
            .inventory
            .lock()
            .expect("inventory lock poisoned")
            .power_upgrades
            .clone();

        let mut total_trigger_count = 0;
        let mut total_reward = 0;

        for item_id in owned_power_upgrade_ids {
            let Some(item) = find_shop_item(&item_id) else {
                continue;
            };
            let Some(effect) = item.power_upgrade_effect else {
                continue;
            };

            let trigger_count =
                self.roll_power_upgrade_triggers(input_count, effect.chance_per_million_inputs);

            if trigger_count == 0 {
                continue;
            }

            if item.id == CRUMPLED_COURT_ONBOARDING_MANUAL_ID {
                self.crumpled_court_onboarding_manual_trigger_count
                    .fetch_add(trigger_count, Ordering::Relaxed);
            }

            total_trigger_count += trigger_count;
            total_reward += trigger_count * effect.reward;
        }

        if total_trigger_count == 0 {
            return;
        }

        self.bonus_influence
            .fetch_add(total_reward, Ordering::Relaxed);
        self.power_event_sequence
            .fetch_add(total_trigger_count, Ordering::Relaxed);
        self.last_power_event_at_millis
            .store(input_at_millis, Ordering::Relaxed);
        self.last_power_event_amount
            .store(total_reward, Ordering::Relaxed);
        self.mark_dirty();
    }

    #[cfg(test)]
    fn process_power_upgrade_rolls_with_effect(
        &self,
        input_at_millis: u64,
        input_count: u64,
        effect: PowerUpgradeEffect,
    ) {
        let trigger_count =
            self.roll_power_upgrade_triggers(input_count, effect.chance_per_million_inputs);

        if trigger_count == 0 {
            return;
        }

        let reward = trigger_count * effect.reward;

        self.bonus_influence.fetch_add(reward, Ordering::Relaxed);
        self.power_event_sequence
            .fetch_add(trigger_count, Ordering::Relaxed);
        self.last_power_event_at_millis
            .store(input_at_millis, Ordering::Relaxed);
        self.last_power_event_amount
            .store(reward, Ordering::Relaxed);
        self.mark_dirty();
    }

    fn roll_power_upgrade_triggers(&self, input_count: u64, chance_per_million_inputs: u64) -> u64 {
        let mut trigger_count = 0;

        for _ in 0..input_count {
            if self.next_random_roll() < chance_per_million_inputs {
                trigger_count += 1;
            }
        }

        trigger_count
    }

    fn next_random_roll(&self) -> u64 {
        loop {
            let current_state = self.random_state.load(Ordering::Relaxed);
            let seeded_state = if current_state == 0 {
                current_time_millis() | 1
            } else {
                current_state
            };
            let next_state = next_random_state(seeded_state);

            if self
                .random_state
                .compare_exchange(
                    current_state,
                    next_state,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return next_state % CHANCE_SCALE;
            }
        }
    }

    fn record_input_time_at(&self, input_at_millis: u64) {
        self.last_input_at_millis
            .store(input_at_millis, Ordering::Relaxed);
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn take_dirty(&self) -> bool {
        self.dirty.swap(false, Ordering::Relaxed)
    }
}

fn level_for_xp(xp: u64) -> u64 {
    let mut level = 1;

    while xp >= xp_required_for_level(level + 1) {
        level += 1;
    }

    level
}

fn xp_required_for_level(level: u64) -> u64 {
    match level {
        0 | 1 => 0,
        2 => 500,
        3 => 1_500,
        4 => 6_000,
        5 => 30_000,
        _ => {
            let mut xp = 30_000;

            for completed_level in 5..level {
                xp += xp_needed_to_advance_from_level(completed_level);
            }

            xp
        }
    }
}

fn xp_needed_to_advance_from_level(level: u64) -> u64 {
    let mut xp_needed = 27_600;

    for _ in 5..level {
        xp_needed = (xp_needed * 115) / 100;
    }

    xp_needed
}

fn find_shop_item(item_id: &str) -> Option<&'static ShopItem> {
    SHOP_ITEMS.iter().find(|item| item.id == item_id)
}

fn next_random_state(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[tauri::command]
fn get_input_counts(counts: tauri::State<'_, Arc<InputCounts>>) -> InputSnapshot {
    counts.snapshot()
}

#[tauri::command]
fn get_runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        presentation_mode: std::env::var("KNIGHT_SHIFT_PRESENTATION_MODE")
            .unwrap_or_else(|_| "gui".to_string()),
    }
}

#[tauri::command]
fn record_focused_keypress(counts: tauri::State<'_, Arc<InputCounts>>, event_at_millis: u64) {
    counts.record_focused_keypress(event_at_millis);
}

#[tauri::command]
fn purchase_shop_item(
    counts: tauri::State<'_, Arc<InputCounts>>,
    item_id: String,
) -> PurchaseResult {
    counts.purchase_shop_item(&item_id)
}

#[tauri::command]
fn reset_progress(counts: tauri::State<'_, Arc<InputCounts>>) -> InputSnapshot {
    counts.reset_progress()
}

#[tauri::command]
fn dev_add_influence(counts: tauri::State<'_, Arc<InputCounts>>, amount: u64) -> InputSnapshot {
    if cfg!(debug_assertions) {
        counts.dev_add_influence(amount)
    } else {
        counts.snapshot()
    }
}

#[tauri::command]
fn exit_app(app: tauri::AppHandle, counts: tauri::State<'_, Arc<InputCounts>>) {
    if let Ok(save_path) = app
        .path()
        .app_data_dir()
        .map(|path| path.join(SAVE_FILE_NAME))
    {
        if let Err(error) = save_counts_to_path(&counts, &save_path) {
            eprintln!("exit save failed: {error}");
        }
    }

    app.exit(0);
}

fn load_save_file(path: &Path) -> Option<SaveData> {
    let save_json = fs::read_to_string(path).ok()?;
    let save_data = serde_json::from_str::<SaveData>(&save_json).ok()?;

    (save_data.version == SAVE_VERSION).then_some(save_data)
}

fn save_counts_to_path(
    counts: &InputCounts,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let save_json = serde_json::to_string_pretty(&counts.snapshot_save())?;
    let temp_path = path.with_extension("json.tmp");

    fs::write(&temp_path, save_json)?;

    if path.exists() {
        fs::remove_file(path)?;
    }

    fs::rename(temp_path, path)?;

    Ok(())
}

fn start_input_listener(counts: Arc<InputCounts>) {
    thread::spawn(move || {
        let callback = move |event: rdev::Event| {
            counts.record_event(event.event_type);
        };

        if let Err(error) = listen(callback) {
            eprintln!("global input listener stopped: {error:?}");
        }
    });
}

fn start_autosave(counts: Arc<InputCounts>, save_path: PathBuf) {
    thread::spawn(move || loop {
        thread::sleep(AUTOSAVE_INTERVAL);

        if !counts.take_dirty() {
            continue;
        }

        if let Err(error) = save_counts_to_path(&counts, &save_path) {
            counts.mark_dirty();
            eprintln!("autosave failed: {error}");
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let input_counts = Arc::new(InputCounts::default());
    let app_settings = Arc::new(SettingsState::default());
    let setup_counts = input_counts.clone();
    let setup_settings = app_settings.clone();
    let close_counts = input_counts.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(input_counts)
        .manage(app_settings)
        .setup(move |app| {
            let save_path = app.path().app_data_dir()?.join(SAVE_FILE_NAME);
            let settings_path = app.path().app_data_dir()?.join(SETTINGS_FILE_NAME);
            let settings = load_settings_file(&settings_path);

            if let Some(save_data) = load_save_file(&save_path) {
                setup_counts.load_save(save_data);
            }

            setup_settings.load(settings.clone());
            if let Err(error) = apply_app_settings(app.handle(), &settings) {
                eprintln!("settings apply failed: {error}");
            }

            start_input_listener(setup_counts.clone());
            start_autosave(setup_counts.clone(), save_path);

            Ok(())
        })
        .on_window_event(move |_window, event| {
            if matches!(event, WindowEvent::CloseRequested { .. }) {
                if let Ok(save_path) = _window
                    .path()
                    .app_data_dir()
                    .map(|path| path.join(SAVE_FILE_NAME))
                {
                    if let Err(error) = save_counts_to_path(&close_counts, &save_path) {
                        eprintln!("final save failed: {error}");
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            dev_add_influence,
            exit_app,
            get_app_settings,
            get_input_counts,
            get_runtime_config,
            purchase_shop_item,
            reset_progress,
            record_focused_keypress,
            update_app_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdev::{Button, Key};

    #[test]
    fn input_counts_track_only_key_presses_and_mouse_presses() {
        let counts = InputCounts::default();

        counts.record_event(EventType::KeyPress(Key::KeyA));
        counts.record_event(EventType::ButtonPress(Button::Left));
        counts.record_event(EventType::MouseMove { x: 10.0, y: 10.0 });
        counts.record_event(EventType::KeyRelease(Key::KeyA));

        let snapshot = counts.snapshot();

        assert_eq!(snapshot.keys, 1);
        assert_eq!(snapshot.clicks, 1);
        assert_eq!(snapshot.influence, 2);
        assert_eq!(snapshot.xp, 2);
        assert_eq!(snapshot.level, 1);
        assert_eq!(snapshot.xp_for_current_level, 0);
        assert_eq!(snapshot.xp_for_next_level, 500);
        assert!(snapshot.last_input_at_millis > 0);
    }

    #[test]
    fn focused_keypresses_are_counted_when_global_hook_misses_them() {
        let counts = InputCounts::default();

        counts.record_focused_keypress(1_000);

        let snapshot = counts.snapshot();

        assert_eq!(snapshot.keys, 1);
        assert_eq!(snapshot.clicks, 0);
        assert_eq!(snapshot.influence, 1);
        assert_eq!(snapshot.xp, 1);
        assert_eq!(snapshot.level, 1);
        assert_eq!(snapshot.xp_for_current_level, 0);
        assert_eq!(snapshot.xp_for_next_level, 500);
        assert_eq!(snapshot.last_input_at_millis, 1_000);
    }

    #[test]
    fn focused_keypresses_are_not_double_counted_when_global_hook_reports_them() {
        let counts = InputCounts::default();

        counts.record_event(EventType::KeyPress(Key::KeyA));
        let global_key_at_millis = counts.last_global_key_at_millis.load(Ordering::Relaxed);
        counts.record_focused_keypress(global_key_at_millis);

        let snapshot = counts.snapshot();

        assert_eq!(snapshot.keys, 1);
        assert_eq!(snapshot.influence, 1);
    }

    #[test]
    fn save_file_round_trips_key_and_click_totals() {
        let path = std::env::temp_dir().join(format!(
            "knight-shift-save-test-{}.json",
            current_time_millis()
        ));
        let counts = InputCounts::default();

        counts.record_event(EventType::KeyPress(Key::KeyA));
        counts.record_event(EventType::ButtonPress(Button::Left));
        save_counts_to_path(&counts, &path).expect("save should write");

        let loaded = load_save_file(&path).expect("save should load");

        assert_eq!(
            loaded,
            SaveData {
                version: SAVE_VERSION,
                keys: 1,
                clicks: 1,
                bonus_influence: 0,
                spent_influence: 0,
                crumpled_court_onboarding_manual_input_baseline: 0,
                crumpled_court_onboarding_manual_trigger_count: 0,
                pending_power_proc_inputs: 0,
                last_power_proc_roll_at_millis: 0,
                inventory: Inventory::default(),
            }
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn purchasing_royal_contract_spends_influence_and_adds_org_chart_inventory() {
        let counts = InputCounts::default();

        for index in 0..500 {
            counts.record_focused_keypress(1_000 + index);
        }

        let result = counts.purchase_shop_item(ROYAL_CONTRACT_ID);

        assert!(matches!(result.status, PurchaseStatus::Purchased));
        assert_eq!(result.snapshot.influence, 400);
        assert_eq!(result.snapshot.xp, 500);
        assert_eq!(result.snapshot.level, 2);
        assert_eq!(result.snapshot.keys, 500);
        assert_eq!(
            result.snapshot.inventory_item_ids,
            vec![ROYAL_CONTRACT_ID.to_string()]
        );

        let save_data = counts.snapshot_save();

        assert_eq!(
            save_data.spent_influence,
            find_shop_item(ROYAL_CONTRACT_ID)
                .expect("royal contract exists")
                .cost
        );
        assert_eq!(save_data.inventory.org_chart, vec![ROYAL_CONTRACT_ID]);
    }

    #[test]
    fn power_upgrade_purchase_is_level_gated_and_stored_separately() {
        let counts = InputCounts::default();

        let locked_result = counts.purchase_shop_item("crumpled_court_onboarding_manual");

        assert!(matches!(locked_result.status, PurchaseStatus::Locked));
        assert!(locked_result.snapshot.inventory_item_ids.is_empty());

        counts.dev_add_influence(1_500);
        let contract_result = counts.purchase_shop_item(ROYAL_CONTRACT_ID);
        let purchased_result = counts.purchase_shop_item("crumpled_court_onboarding_manual");
        let save_data = counts.snapshot_save();

        assert!(matches!(contract_result.status, PurchaseStatus::Purchased));
        assert!(matches!(purchased_result.status, PurchaseStatus::Purchased));
        assert_eq!(purchased_result.snapshot.influence, 1_300);
        assert_eq!(
            purchased_result.snapshot.inventory_item_ids,
            vec![
                ROYAL_CONTRACT_ID.to_string(),
                "crumpled_court_onboarding_manual".to_string()
            ]
        );
        assert_eq!(save_data.inventory.org_chart, vec![ROYAL_CONTRACT_ID]);
        assert_eq!(
            save_data.inventory.power_upgrades,
            vec!["crumpled_court_onboarding_manual"]
        );
    }

    #[test]
    fn purchase_fails_without_enough_influence() {
        let counts = InputCounts::default();

        let result = counts.purchase_shop_item(ROYAL_CONTRACT_ID);

        assert!(matches!(result.status, PurchaseStatus::Locked));
        assert_eq!(result.snapshot.influence, 0);
        assert_eq!(result.snapshot.xp, 0);
        assert_eq!(result.snapshot.level, 1);
        assert_eq!(result.snapshot.xp_for_current_level, 0);
        assert_eq!(result.snapshot.xp_for_next_level, 500);
        assert!(result.snapshot.inventory_item_ids.is_empty());
    }

    #[test]
    fn level_curve_uses_early_manual_thresholds_then_steady_growth() {
        assert_eq!(level_for_xp(0), 1);
        assert_eq!(level_for_xp(499), 1);
        assert_eq!(level_for_xp(500), 2);
        assert_eq!(level_for_xp(1_499), 2);
        assert_eq!(level_for_xp(1_500), 3);
        assert_eq!(level_for_xp(5_999), 3);
        assert_eq!(level_for_xp(6_000), 4);
        assert_eq!(level_for_xp(29_999), 4);
        assert_eq!(level_for_xp(30_000), 5);
        assert!(xp_needed_to_advance_from_level(6) > xp_needed_to_advance_from_level(5));
        assert!(xp_needed_to_advance_from_level(10) > xp_needed_to_advance_from_level(6));
    }

    #[test]
    fn dev_add_influence_changes_earned_state_without_spending() {
        let counts = InputCounts::default();

        let snapshot = counts.dev_add_influence(500);
        let save_data = counts.snapshot_save();

        assert_eq!(snapshot.influence, 500);
        assert_eq!(snapshot.xp, 500);
        assert_eq!(snapshot.level, 2);
        assert_eq!(snapshot.keys, 500);
        assert_eq!(snapshot.clicks, 0);
        assert_eq!(snapshot.bonus_influence, 0);
        assert_eq!(save_data.keys, 500);
        assert_eq!(save_data.bonus_influence, 0);
        assert_eq!(save_data.spent_influence, 0);
    }

    #[test]
    fn crumpled_court_onboarding_manual_tracks_random_proc_state_after_purchase() {
        let counts = InputCounts::default();

        counts.dev_add_influence(1_500);
        let contract_result = counts.purchase_shop_item(ROYAL_CONTRACT_ID);
        let purchased_result = counts.purchase_shop_item(CRUMPLED_COURT_ONBOARDING_MANUAL_ID);

        assert!(matches!(contract_result.status, PurchaseStatus::Purchased));
        assert!(matches!(purchased_result.status, PurchaseStatus::Purchased));
        assert_eq!(purchased_result.snapshot.influence, 1_300);
        assert_eq!(purchased_result.snapshot.bonus_influence, 0);
        assert_eq!(purchased_result.snapshot.power_event_sequence, 0);

        let save_data = counts.snapshot_save();

        assert_eq!(save_data.bonus_influence, 0);
        assert_eq!(save_data.crumpled_court_onboarding_manual_trigger_count, 0);
        assert_eq!(
            save_data.crumpled_court_onboarding_manual_input_baseline,
            1_500
        );
    }

    #[test]
    fn random_power_upgrade_rolls_can_trigger_bonus_rewards() {
        let counts = InputCounts::default();

        counts.load_save(SaveData {
            version: SAVE_VERSION,
            keys: 1_500,
            clicks: 0,
            bonus_influence: 0,
            spent_influence: 100,
            crumpled_court_onboarding_manual_input_baseline: 0,
            crumpled_court_onboarding_manual_trigger_count: 0,
            pending_power_proc_inputs: 0,
            last_power_proc_roll_at_millis: 0,
            inventory: Inventory {
                org_chart: Vec::new(),
                power_upgrades: vec![CRUMPLED_COURT_ONBOARDING_MANUAL_ID.to_string()],
            },
        });

        counts.process_power_upgrade_rolls_with_effect(
            1_000,
            3,
            PowerUpgradeEffect::new(1_000_000, 25),
        );
        let triggered = counts.snapshot();

        assert_eq!(triggered.bonus_influence, 75);
        assert_eq!(triggered.power_event_sequence, 3);
        assert_eq!(triggered.last_power_event_amount, 75);
        assert_eq!(triggered.last_power_event_at_millis, 1_000);
    }

    #[test]
    fn all_power_upgrade_catalog_items_have_backend_proc_effects() {
        for item in SHOP_ITEMS {
            match item.category {
                ShopItemCategory::OrgChart => assert!(item.power_upgrade_effect.is_none()),
                ShopItemCategory::PowerUpgrade => {
                    let effect = item
                        .power_upgrade_effect
                        .expect("power upgrade should have an effect");

                    assert!(effect.chance_per_million_inputs > 0);
                    assert!(effect.chance_per_million_inputs <= CHANCE_SCALE);
                    assert!(effect.reward > 0);
                }
            }
        }
    }

    #[test]
    fn proc_rolls_are_batched_until_cooldown_window_elapses() {
        let counts = InputCounts::default();

        counts.load_save(SaveData {
            version: SAVE_VERSION,
            keys: 0,
            clicks: 0,
            bonus_influence: 0,
            spent_influence: 0,
            crumpled_court_onboarding_manual_input_baseline: 0,
            crumpled_court_onboarding_manual_trigger_count: 0,
            pending_power_proc_inputs: 0,
            last_power_proc_roll_at_millis: 0,
            inventory: Inventory {
                org_chart: Vec::new(),
                power_upgrades: vec![CRUMPLED_COURT_ONBOARDING_MANUAL_ID.to_string()],
            },
        });

        counts.queue_power_upgrade_inputs(1_000, 3);
        counts.flush_due_power_upgrades(1_999);

        assert_eq!(counts.pending_power_proc_inputs.load(Ordering::Relaxed), 3);
        assert_eq!(counts.power_event_sequence.load(Ordering::Relaxed), 0);

        assert_eq!(counts.bonus_influence.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn reset_progress_clears_counts_spending_and_inventory() {
        let counts = InputCounts::default();

        for index in 0..100 {
            counts.record_focused_keypress(1_000 + index);
        }

        counts.purchase_shop_item(ROYAL_CONTRACT_ID);
        counts.record_event(EventType::ButtonPress(Button::Left));

        let snapshot = counts.reset_progress();
        let save_data = counts.snapshot_save();

        assert_eq!(snapshot.influence, 0);
        assert_eq!(snapshot.xp, 0);
        assert_eq!(snapshot.level, 1);
        assert_eq!(snapshot.xp_for_current_level, 0);
        assert_eq!(snapshot.xp_for_next_level, 500);
        assert_eq!(snapshot.keys, 0);
        assert_eq!(snapshot.clicks, 0);
        assert_eq!(snapshot.bonus_influence, 0);
        assert_eq!(snapshot.power_event_sequence, 0);
        assert!(snapshot.inventory_item_ids.is_empty());
        assert_eq!(save_data.spent_influence, 0);
        assert_eq!(save_data.bonus_influence, 0);
        assert_eq!(save_data.crumpled_court_onboarding_manual_input_baseline, 0);
        assert_eq!(save_data.crumpled_court_onboarding_manual_trigger_count, 0);
        assert_eq!(save_data.inventory, Inventory::default());
    }
}
