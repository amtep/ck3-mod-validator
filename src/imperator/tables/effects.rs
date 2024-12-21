use once_cell::sync::Lazy;

use crate::effect::Effect;
use crate::effect_validation::*;
use crate::everything::Everything;
use crate::helpers::TigerHashMap;
use crate::imperator::effect_validation::*;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;
use Effect::*;

pub fn scope_effect(name: &Token, _data: &Everything) -> Option<(Scopes, Effect)> {
    let name_lc = name.as_str().to_ascii_lowercase();
    SCOPE_EFFECT_MAP.get(&*name_lc).copied()
}

/// A hashed version of [`SCOPE_EFFECT`], for quick lookup by effect name.
static SCOPE_EFFECT_MAP: Lazy<TigerHashMap<&'static str, (Scopes, Effect)>> = Lazy::new(|| {
    let mut hash = TigerHashMap::default();
    for (from, s, effect) in SCOPE_EFFECT.iter().copied() {
        hash.insert(s, (from, effect));
    }
    hash
});

// LAST UPDATED VERSION 2.0.4
// See `effects.log` from the game data dumps
// Note: There are a lot of effects here that are marked as "Unchecked"
// Most of these are actually deprecated OR have no example usage so can't really be checked properly
const SCOPE_EFFECT: &[(Scopes, &str, Effect)] = &[
    (Scopes::State, "add_trade_route", Vb(validate_trade_route)),
    (Scopes::State, "remove_trade_route", Vb(validate_trade_route)),
    (Scopes::State, "set_automated_trading", Boolean),
    (Scopes::State, "set_governor_policy", Item(Item::GovernorPolicy)),
    (Scopes::State, "add_state_food", ScriptValue),
    (Scopes::State, "add_state_modifier", Vbv(validate_add_modifier)),
    (Scopes::State, "remove_state_modifier", Item(Item::Modifier)),
    (Scopes::State, "set_state_capital", ScopeOrItem(Scopes::Province, Item::Province)),
    (Scopes::Character, "adapt_family_name", Boolean),
    (Scopes::Character, "add_as_governor", Scope(Scopes::Governorship)),
    (Scopes::Character, "add_character_experience", ScriptValue),
    (Scopes::Character, "add_character_modifier", Vbv(validate_add_modifier)),
    (Scopes::Character, "add_corruption", ScriptValue),
    (Scopes::Character, "add_friend", Scope(Scopes::Character)),
    (Scopes::Character, "add_gold", ScriptValue),
    (Scopes::Character, "add_health", ScriptValue),
    (Scopes::Character, "add_holding", ScopeOrItem(Scopes::Province, Item::Province)),
    (Scopes::Character, "add_loyal_veterans", ScriptValue),
    (Scopes::Character, "add_loyal_veterans", ScriptValue),
    (Scopes::Character, "add_loyalty", Item(Item::Loyalty)),
    (Scopes::Character, "add_nickname", Item(Item::Localization)),
    (Scopes::Character, "add_party_conviction", Vb(validate_add_party_conviction_or_approval)),
    (Scopes::Character, "add_popularity", ScriptValue),
    (Scopes::Character, "add_prominence", ScriptValue),
    (Scopes::Character, "add_rival", Scope(Scopes::Character)),
    (Scopes::Character, "remove_rival", Scope(Scopes::Character)),
    (Scopes::Character, "add_ruler_conviction", Removed("2.0", "")),
    (Scopes::Character, "add_trait", Item(Item::CharacterTrait)),
    (Scopes::Character, "add_triggered_character_modifier", Vbv(validate_add_modifier)),
    (Scopes::Character, "adopt", Scope(Scopes::Character)),
    (Scopes::Character, "banish", ScopeOrItem(Scopes::Country, Item::Localization)),
    (
        Scopes::Character,
        "change_mercenary_employer",
        ScopeOrItem(Scopes::Country, Item::Localization),
    ),
    (Scopes::Character, "clear_ambition", Boolean),
    (Scopes::Character, "death", Vb(validate_death)),
    (Scopes::Character, "deify_character", Vb(validate_deify_character)),
    (Scopes::Character, "divorce_character", Scope(Scopes::Character)),
    (Scopes::Character, "end_pregnancy", Boolean),
    (Scopes::Character, "force_add_trait", Item(Item::CharacterTrait)),
    (Scopes::Character, "give_office", Item(Item::Office)),
    (Scopes::Character, "marry_character", Scope(Scopes::Character)),
    (Scopes::Character, "move_country", ScopeOrItem(Scopes::Country, Item::Localization)),
    (
        Scopes::Character,
        "move_country_with_message",
        ScopeOrItem(Scopes::Country, Item::Localization),
    ),
    (Scopes::Character, "pay_gold", Vb(validate_pay_gold)),
    (Scopes::Character, "remove_all_offices", Boolean),
    (Scopes::Character, "remove_as_governor", Boolean),
    (Scopes::Character, "remove_as_mercenary", Boolean),
    (Scopes::Character, "remove_as_researcher", Boolean),
    (Scopes::Character, "remove_character_modifier", Item(Item::Modifier)),
    (Scopes::Character, "remove_command", Boolean),
    (Scopes::Character, "remove_friend", Scope(Scopes::Character)),
    (Scopes::Character, "remove_holding", ScopeOrItem(Scopes::Province, Item::Province)),
    (Scopes::Character, "remove_loyalty", Item(Item::Loyalty)),
    (Scopes::Character, "remove_office", Item(Item::Office)),
    (Scopes::Character, "remove_trait", Item(Item::CharacterTrait)),
    (Scopes::Character, "remove_triggered_character_modifier", Item(Item::Modifier)),
    (Scopes::Character, "set_ambition", Item(Item::Ambition)),
    (Scopes::Character, "set_as_minor_character", ScopeOkThis(Scopes::Character)),
    (Scopes::Character, "set_character_religion", ScopeOrItem(Scopes::Religion, Item::Religion)),
    (Scopes::Character, "set_culture", ScopeOrItem(Scopes::Culture, Item::Culture)),
    (Scopes::Character, "set_culture_same_as", ScopeOrItem(Scopes::Culture, Item::Culture)),
    (Scopes::Character, "set_family", Scope(Scopes::Family)),
    (Scopes::Character, "set_firstname", Item(Item::Localization)),
    (Scopes::Character, "set_home_country", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Character, "set_party_leader", Unchecked),
    (Scopes::Character, "update_character", Boolean),
    (
        Scopes::Character.union(Scopes::Unit).union(Scopes::Legion),
        "add_legion_history",
        Vb(validate_legion_history),
    ),
    (Scopes::Character.union(Scopes::Unit), "add_to_legion", Scope(Scopes::Legion)),
    (Scopes::Character, "add_charisma", ScriptValue),
    (Scopes::Character, "add_finesse", ScriptValue),
    (Scopes::Character, "add_martial", ScriptValue),
    (Scopes::Character, "add_zeal", ScriptValue),
    (Scopes::Character, "make_pregnant", Vb(validate_make_pregnant)),
    (Scopes::Governorship, "disband_legion", Boolean),
    (Scopes::Governorship, "raise_legion", Vb(validate_raise_legion)),
    (Scopes::Treasure, "destroy_treasure", Boolean),
    (Scopes::Treasure, "transfer_treasure_to_character", Scope(Scopes::Character)),
    (
        Scopes::Treasure,
        "transfer_treasure_to_country",
        ScopeOrItem(Scopes::Country, Item::Localization),
    ),
    (
        Scopes::Treasure,
        "transfer_treasure_to_province",
        ScopeOrItem(Scopes::Province, Item::Province),
    ),
    (Scopes::Country, "add_deity_to_pantheon", Vb(validate_add_deity_to_pantheon)),
    (Scopes::Country, "play_sound_effect", Item(Item::Sound)),
    (Scopes::Country, "set_antagonist", Boolean),
    (Scopes::Country, "set_player_country", Scope(Scopes::Country)),
    (Scopes::Country, "unlock_invention", Item(Item::Invention)),
    (Scopes::Country, "add_aggressive_expansion", ScriptValue),
    (Scopes::Country, "add_alliance", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Country, "add_centralization", ScriptValue),
    (Scopes::Country, "add_country_modifier", Vbv(validate_add_modifier)),
    (Scopes::Country, "add_guarantee", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Country, "add_innovation", ScriptValue),
    (Scopes::Country, "add_legitimacy", ScriptValue),
    (Scopes::Country, "add_manpower", ScriptValue),
    (Scopes::Country, "add_military_access", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Country, "add_military_experience", ScriptValue),
    (Scopes::Country, "add_new_family", Item(Item::Localization)),
    (Scopes::Country, "add_opinion", Vb(validate_change_opinion)),
    (Scopes::Country, "add_party_approval", Vb(validate_add_party_conviction_or_approval)),
    (Scopes::Country, "add_political_influence", ScriptValue),
    (Scopes::Country, "add_research", Vb(validate_add_research)),
    (Scopes::Country, "add_stability", ScriptValue),
    (Scopes::Country, "add_to_war", Vb(validate_add_to_war)),
    (Scopes::Country, "add_treasury", ScriptValue),
    (Scopes::Country, "add_truce", Vb(validate_add_truce)),
    (Scopes::Country, "add_tyranny", ScriptValue),
    (Scopes::Country, "add_war_exhaustion", ScriptValue),
    (Scopes::Country, "change_country_adjective", Item(Item::Localization)),
    (Scopes::Country, "change_country_color", Item(Item::NamedColor)),
    (Scopes::Country, "change_country_flag", Item(Item::Coa)),
    (Scopes::Country, "change_country_name", Item(Item::Localization)),
    (Scopes::Country, "change_country_tag", Item(Item::Localization)),
    (Scopes::Country, "change_government", Item(Item::GovernmentType)),
    (Scopes::Country, "change_law", Item(Item::Law)),
    (Scopes::Country, "create_character", Vb(validate_create_character)),
    (Scopes::Country, "create_country_treasure", Vb(validate_create_treasure)),
    (Scopes::Country, "create_family", Scope(Scopes::Character)),
    (Scopes::Country, "declare_war_with_wargoal", Vb(validate_declare_war)),
    (Scopes::Country, "imprison", Vb(validate_imprison)),
    (Scopes::Country, "integrate_country_culture", Scope(Scopes::CountryCulture)),
    (Scopes::Country, "make_subject", Vb(validate_make_subject)),
    (Scopes::Country, "pay_price", Item(Item::Price)),
    (Scopes::Country, "recalc_succession", Boolean),
    (Scopes::Country, "refund_price", Item(Item::Price)),
    (Scopes::Country, "release_prisoner", Vbv(validate_release_prisoner)),
    (Scopes::Country, "release_subject", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Country, "remove_country_modifier", Item(Item::Modifier)),
    (Scopes::Country, "remove_gurantee", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Country, "remove_opinion", Vb(validate_change_opinion)),
    (Scopes::Country, "remove_party_leadership", Scope(Scopes::Party)),
    (Scopes::Country, "reverse_add_opinion", Vb(validate_change_opinion)),
    (Scopes::Country, "set_as_coruler", Scope(Scopes::Character)),
    (Scopes::Country, "set_as_ruler", Scope(Scopes::Character)),
    (Scopes::Country, "set_capital", ScopeOrItem(Scopes::Province, Item::Province)),
    (Scopes::Country, "set_country_heritage", Item(Item::Heritage)),
    (Scopes::Country, "set_country_religion", ScopeOrItem(Scopes::Religion, Item::Religion)),
    (Scopes::Country, "set_gender_equality", Boolean),
    (Scopes::Country, "set_graphical_culture", Item(Item::GraphicalCultureType)),
    (Scopes::Country, "set_ignore_senate_approval", Boolean),
    (Scopes::Country, "set_legion_recruitment", Choice(&["enabled", "disabled", "capital"])),
    (Scopes::Country, "set_primary_culture", ScopeOrItem(Scopes::Culture, Item::Culture)),
    (Scopes::Country, "start_civil_war", Scope(Scopes::Character)),
    (Scopes::Country, "update_allowed_parties", Boolean),
    (Scopes::Country, "set_party_agenda", Unchecked),
    (Scopes::Country, "break_alliance", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Legion, "add_commander", Scope(Scopes::Character)),
    (Scopes::Legion, "add_distinction", Item(Item::LegionDistinction)),
    (Scopes::Legion, "add_legion_unit", Unchecked),
    (Scopes::Legion, "move_legion", Scope(Scopes::Governorship)),
    (Scopes::Legion, "remove_commander", Scope(Scopes::Character)),
    (Scopes::Legion, "remove_distinction", Item(Item::LegionDistinction)),
    (Scopes::Legion, "remove_legion_unit", Unchecked),
    (Scopes::Siege, "add_breach", Integer),
    (Scopes::Legion.union(Scopes::Country), "create_unit", Vb(validate_create_unit)),
    (Scopes::Unit, "add_food", ScriptValue),
    (Scopes::Unit, "add_loyal_subunit", Item(Item::Unit)),
    (Scopes::Unit, "add_morale", ScriptValue),
    (Scopes::Unit, "add_subunit", Item(Item::Unit)),
    (Scopes::Unit, "add_unit_modifier", Vbv(validate_add_modifier)),
    (Scopes::Unit, "change_unit_owner", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Unit, "damage_unit_morale_percent", ScriptValue),
    (Scopes::Unit, "damage_unit_percent", ScriptValue),
    (Scopes::Unit, "destroy_unit", Boolean),
    (Scopes::Unit, "lock_unit", ScriptValue),
    (Scopes::Unit, "unlock_unit", ScriptValue),
    (Scopes::Unit, "remove_unit_loyalty", Boolean),
    (Scopes::Unit, "remove_unit_modifier", Item(Item::Modifier)),
    (Scopes::Unit, "set_as_commander", Scope(Scopes::Character)),
    (Scopes::Unit, "set_unit_size", Unchecked),
    (Scopes::Unit, "split_migrants_to", ScriptValue),
    (Scopes::Party, "pick_random_agenda", Boolean),
    (Scopes::Pop, "kill_pop", Boolean),
    (Scopes::Pop, "move_pop", ScopeOrItem(Scopes::Province, Item::Province)),
    (Scopes::Pop, "set_pop_culture", ScopeOrItem(Scopes::Culture, Item::Culture)),
    (Scopes::Pop, "set_pop_culture_same_as", Scope(Scopes::Pop)),
    (Scopes::Pop, "set_pop_religion", ScopeOrItem(Scopes::Religion, Item::Religion)),
    (Scopes::Pop, "set_pop_religion_same_as", Scope(Scopes::Pop)),
    (Scopes::Pop, "set_pop_type", Item(Item::PopType)),
    (Scopes::SubUnit, "add_subunit_morale", ScriptValue),
    (Scopes::SubUnit, "add_subunit_strength", ScriptValue),
    (Scopes::SubUnit, "destroy_subunit", Boolean),
    (Scopes::SubUnit, "remove_subunit_loyalty", Vv(validate_remove_subunit_loyalty)),
    (Scopes::SubUnit, "set_personal_loyalty", Scope(Scopes::Character)),
    (Scopes::Family, "add_prestige", ScriptValue),
    (Scopes::Family, "move_family", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Family, "remove_family", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Country.union(Scopes::War), "force_white_peace", Scope(Scopes::War)),
    (Scopes::War, "remove_from_war", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Province, "add_building_level", Item(Item::Building)),
    (Scopes::Province, "add_civilization_value", ScriptValue),
    (Scopes::Province, "add_claim", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Province, "add_permanent_province_modifier", Vbv(validate_add_modifier)),
    (Scopes::Province, "add_province_modifier", Vbv(validate_add_modifier)),
    (Scopes::Province, "add_road_towards", ScopeOrItem(Scopes::Province, Item::Province)),
    (Scopes::Province, "add_state_loyalty", ScriptValue),
    (Scopes::Province, "add_vfx", Unchecked),
    (Scopes::Province, "begin_great_work_construction", Vb(validate_great_work_construction)),
    (Scopes::Province, "change_province_name", Item(Item::Localization)),
    (Scopes::Province, "create_country", Vb(validate_create_country)),
    (Scopes::Province, "create_pop", Item(Item::PopType)),
    (Scopes::Province, "create_state_pop", Item(Item::PopType)),
    (Scopes::Province, "define_pop", Vb(validate_define_pop)),
    (Scopes::Province, "finish_great_work_construction", Vb(validate_great_work_construction)),
    (Scopes::Province, "hide_model", Unchecked),
    (Scopes::Province, "remove_building_level", Item(Item::Building)),
    (Scopes::Province, "remove_claim", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Province, "remove_province_deity", Boolean),
    (Scopes::Province, "remove_province_modifier", Item(Item::Modifier)),
    (Scopes::Province, "remove_vfx", Unchecked),
    (Scopes::Province, "set_as_governor", Scope(Scopes::Character)),
    (Scopes::Province, "set_city_status", Item(Item::ProvinceRank)),
    (Scopes::Province, "set_conquered_by", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Province, "set_controller", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Province, "set_owned_by", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::Province, "set_province_deity", Scope(Scopes::Deity)),
    (Scopes::Province, "set_trade_goods", Item(Item::TradeGood)),
    (Scopes::Province, "show_animated_text", Unchecked),
    (Scopes::Province, "show_model", Unchecked),
    (Scopes::CountryCulture, "add_country_culture_modifier", Vbv(validate_add_modifier)),
    (Scopes::CountryCulture, "add_integration_progress", ScriptValue),
    (Scopes::CountryCulture, "remove_country_culture_modifier", Item(Item::Modifier)),
    (Scopes::CountryCulture, "set_country_culture_right", Item(Item::PopType)),
    (Scopes::None, "reset_scoring", ScopeOrItem(Scopes::Country, Item::Localization)),
    (Scopes::None, "add_to_global_variable_list", Vb(validate_add_to_variable_list)),
    (Scopes::all_but_none(), "add_to_list", Vv(validate_add_to_list)),
    (Scopes::None, "add_to_local_variable_list", Vb(validate_add_to_variable_list)),
    (Scopes::all_but_none(), "add_to_temporary_list", Vv(validate_add_to_list)),
    (Scopes::None, "add_to_variable_list", Vb(validate_add_to_variable_list)),
    (Scopes::None, "assert_if", Unchecked),
    (Scopes::None, "assert_read", Unchecked),
    (Scopes::None, "break", Yes),
    (Scopes::None, "break_if", Control),
    (Scopes::None, "change_global_variable", Vb(validate_change_variable)),
    (Scopes::None, "change_local_variable", Vb(validate_change_variable)),
    (Scopes::None, "change_variable", Vb(validate_change_variable)),
    (Scopes::None, "clamp_global_variable", Vb(validate_clamp_variable)),
    (Scopes::None, "clamp_local_variable", Vb(validate_clamp_variable)),
    (Scopes::None, "clamp_variable", Vb(validate_clamp_variable)),
    (Scopes::None, "clear_global_variable_list", Unchecked),
    (Scopes::None, "clear_local_variable_list", Unchecked),
    (Scopes::None, "clear_saved_scope", Unchecked),
    (Scopes::None, "clear_variable_list", Unchecked),
    (Scopes::None, "custom_label", ControlOrLabel),
    (Scopes::None, "custom_tooltip", ControlOrLabel),
    (Scopes::None, "debug_log", Unchecked),
    (Scopes::None, "debug_log_scopes", Boolean),
    (Scopes::None, "else", Control),
    (Scopes::None, "else_if", Control),
    (Scopes::None, "hidden_effect", Control),
    (Scopes::None, "if", Control),
    (Scopes::None, "random", Control),
    (Scopes::None, "random_list", Vb(validate_random_list)),
    (Scopes::all_but_none(), "remove_from_list", Vv(validate_remove_from_list)),
    (Scopes::None, "remove_global_variable", Unchecked),
    (Scopes::None, "remove_list_global_variable", Vb(validate_add_to_variable_list)),
    (Scopes::None, "remove_list_local_variable", Vb(validate_add_to_variable_list)),
    (Scopes::None, "remove_list_variable", Vb(validate_add_to_variable_list)),
    (Scopes::None, "remove_local_variable", Unchecked),
    (Scopes::None, "remove_variable", Unchecked),
    (Scopes::None, "round_global_variable", Vb(validate_round_variable)),
    (Scopes::None, "round_local_variable", Vb(validate_round_variable)),
    (Scopes::None, "round_variable", Vb(validate_round_variable)),
    (Scopes::all_but_none(), "save_scope_as", Vv(validate_save_scope)),
    (Scopes::all_but_none(), "save_temporary_scope_as", Vv(validate_save_scope)),
    (Scopes::None, "set_global_variable", Vbv(validate_set_variable)),
    (Scopes::None, "set_local_variable", Vbv(validate_set_variable)),
    (Scopes::None, "set_variable", Vbv(validate_set_variable)),
    (Scopes::None, "show_as_tooltip", Control),
    (Scopes::None, "switch", Vb(validate_switch)),
    (Scopes::None, "trigger_event", Vbv(validate_trigger_event)),
    (Scopes::None, "while", Control),
];
