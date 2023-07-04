/// A module for validation functions that are useful for more than one data module.
use std::fmt::{Display, Formatter};

use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::data::scripted_modifiers::ScriptedModifier;
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::ErrorKey;
use crate::report::{error, error_info, warn};
use crate::scopes::{scope_prefix, scope_to_scope, Scopes};
use crate::scriptvalue::{validate_non_dynamic_scriptvalue, validate_scriptvalue};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{
    validate_normal_trigger, validate_target, validate_target_ok_this, validate_trigger,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ListType {
    None,
    Any,
    Every,
    Ordered,
    Random,
}

impl Display for ListType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ListType::None => write!(f, ""),
            ListType::Any => write!(f, "any"),
            ListType::Every => write!(f, "every"),
            ListType::Ordered => write!(f, "ordered"),
            ListType::Random => write!(f, "random"),
        }
    }
}

impl TryFrom<&str> for ListType {
    type Error = std::fmt::Error;

    fn try_from(from: &str) -> Result<Self, Self::Error> {
        match from {
            "" => Ok(ListType::None),
            "any" => Ok(ListType::Any),
            "every" => Ok(ListType::Every),
            "ordered" => Ok(ListType::Ordered),
            "random" => Ok(ListType::Random),
            _ => Err(Self::Error::default()),
        }
    }
}

pub fn validate_theme_background(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    match bv {
        BV::Value(token) => {
            data.verify_exists(Item::EventBackground, token);
            let block = Block::new(token.loc.clone());
            data.validate_call(Item::EventBackground, token, &block, sc);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);

            vd.field_validated_block("trigger", |b, data| {
                validate_normal_trigger(b, data, sc, Tooltipped::No);
            });
            if vd.field_value("event_background").is_some() {
                let msg = "`event_background` now causes a crash. It has been replaced by `reference` since 1.9";
                error(
                    block.get_key("event_background").unwrap(),
                    ErrorKey::Crash,
                    msg,
                );
            }
            vd.req_field("reference");
            if let Some(token) = vd.field_value("reference") {
                data.verify_exists(Item::EventBackground, token);
                data.validate_call(Item::EventBackground, token, block, sc);
            }
        }
    }
}

pub fn validate_theme_icon(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, Tooltipped::No);
    });
    vd.field_item("reference", Item::File);
}

pub fn validate_theme_sound(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, Tooltipped::No);
    });
    vd.field_item("reference", Item::Sound);
}

pub fn validate_theme_transition(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, Tooltipped::No);
    });
    if let Some(token) = vd.field_value("reference") {
        data.verify_exists(Item::EventTransition, token);
        data.validate_call(Item::EventTransition, token, block, sc);
    }
}

pub fn validate_days_weeks_months_years(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;

    for field in &["days", "weeks", "months", "years"] {
        if let Some(bv) = vd.field_any_cmp(field) {
            validate_scriptvalue(bv, data, sc);
            count += 1;
        }
    }

    if count != 1 {
        let msg = "must have 1 of days, weeks, months, or years";
        let key = if count == 0 {
            ErrorKey::FieldMissing
        } else {
            ErrorKey::Validation
        };
        error(block, key, msg);
    }
}

// Very similar to validate_days_weeks_months_years, but requires = instead of allowing comparators
// "weeks" is not documented but is used all over vanilla TODO: verify
pub fn validate_duration(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;

    for field in &["days", "weeks", "months", "years"] {
        if vd.field_script_value(field, sc) {
            count += 1;
        }
    }

    if count != 1 {
        let msg = "must have 1 of days, weeks, months, or years";
        let key = if count == 0 {
            ErrorKey::FieldMissing
        } else {
            ErrorKey::Validation
        };
        error(block, key, msg);
    }
}

// Very similar to validate_duration, but validates part of a block that may contain a duration
// Also does not accept scriptvalues (per the documentation)
pub fn validate_optional_duration_int(vd: &mut Validator) {
    let mut count = 0;

    for field in &["days", "weeks", "months", "years"] {
        vd.field_validated_value(field, |key, value, _| {
            value.expect_integer();
            count += 1;
            if count > 1 {
                let msg = "must have at most 1 of days, weeks, months, or years";
                error(key, ErrorKey::Validation, msg);
            }
        });
    }
}

// Very similar to validate_days_weeks_months_years, but requires = instead of allowing comparators
pub fn validate_optional_duration(vd: &mut Validator, sc: &mut ScopeContext) {
    let mut count = 0;

    for field in &["days", "weeks", "months", "years"] {
        vd.field_validated_key(field, |key, bv, data| {
            validate_scriptvalue(bv, data, sc);
            count += 1;
            if count > 1 {
                let msg = "must have at most 1 of days, weeks, months, or years";
                error(key, ErrorKey::Validation, msg);
            }
        });
    }
}

// Does not accept scriptvalues (per the documentation)
pub fn validate_color(block: &Block, _data: &Everything) {
    let mut count = 0;
    // Get the color tag, as in color = hsv { 0.5 1.0 1.0 }
    let tag = block.tag.as_ref().map_or("rgb", Token::as_str);
    for (k, _, v) in block.iter_items() {
        if let Some(key) = k {
            error(key, ErrorKey::Colors, "expected color value");
        } else {
            match v {
                BV::Value(t) => {
                    if tag == "hsv" {
                        t.check_number();
                        if let Some(f) = t.get_number() {
                            if !(0.0..=1.0).contains(&f) {
                                let msg = "hsv values should be between 0.0 and 1.0";
                                error(t, ErrorKey::Colors, msg);
                            }
                        } else {
                            error(t, ErrorKey::Colors, "expected hsv value");
                        }
                    } else if tag == "hsv360" {
                        if let Some(i) = t.get_integer() {
                            if !(0..=360).contains(&i) {
                                let msg = "hsv360 values should be between 0 and 360";
                                error(t, ErrorKey::Colors, msg);
                            }
                        } else {
                            error(t, ErrorKey::Colors, "expected hsv360 value");
                        }
                    } else if let Some(i) = t.get_integer() {
                        if !(0..=255).contains(&i) {
                            let msg = "color values should be between 0 and 255";
                            error(t, ErrorKey::Colors, msg);
                        }
                    } else if let Some(f) = t.get_number() {
                        t.check_number();
                        if !(0.0..=1.0).contains(&f) {
                            let msg = "color values should be between 0.0 and 1.0";
                            error(t, ErrorKey::Colors, msg);
                        }
                    } else {
                        error(t, ErrorKey::Colors, "expected color value");
                    }
                    count += 1;
                }
                BV::Block(b) => {
                    error(b, ErrorKey::Colors, "expected color value");
                }
            }
        }
    }
    if count != 3 && count != 4 {
        error(block, ErrorKey::Colors, "expected 3 or 4 color values");
    }
}

/// Camera colors must be hsv, and value can be > 1
pub fn validate_camera_color(block: &Block, data: &Everything) {
    let mut count = 0;
    // Get the color tag, as in color = hsv { 0.5 1.0 1.0 }
    let tag = block.tag.as_ref().map_or("rgb", Token::as_str);
    if tag != "hsv" {
        let msg = "camera colors should be in hsv";
        warn(block, ErrorKey::Colors, msg);
        validate_color(block, data);
        return;
    }

    for (k, _, v) in block.iter_items() {
        if let Some(key) = k {
            error(key, ErrorKey::Colors, "expected color value");
        } else {
            match v {
                BV::Value(t) => {
                    t.check_number();
                    if let Some(f) = t.get_number() {
                        if count <= 1 && !(0.0..=1.0).contains(&f) {
                            let msg = "h and s values should be between 0.0 and 1.0";
                            error(t, ErrorKey::Colors, msg);
                        }
                    } else {
                        error(t, ErrorKey::Colors, "expected hsv value");
                    }
                    count += 1;
                }
                BV::Block(b) => {
                    error(b, ErrorKey::Colors, "expected color value");
                }
            }
        }
    }
    if count != 3 {
        error(block, ErrorKey::Colors, "expected 3 color values");
    }
}

pub fn validate_possibly_named_color(bv: &BV, data: &Everything) {
    match bv {
        BV::Value(token) => data.verify_exists(Item::NamedColor, token),
        BV::Block(block) => validate_color(block, data),
    }
}

pub fn validate_prefix_reference(prefix: &Token, arg: &Token, data: &Everything) {
    // TODO there are more to match
    // TODO integrate this to the SCOPE_FROM_PREFIX table
    match prefix.as_str() {
        "accolade_type" => data.verify_exists(Item::AccoladeType, arg),
        "activity_type" => data.verify_exists(Item::ActivityType, arg),
        "aptitude" | "court_position" => data.verify_exists(Item::CourtPosition, arg),
        "array_define" | "define" => data.verify_exists(Item::Define, arg),
        "character" => data.verify_exists(Item::Character, arg),
        "council_task" | "cp" => data.verify_exists(Item::CouncilPosition, arg),
        "culture" => data.verify_exists(Item::Culture, arg),
        "culture_pillar" => data.verify_exists(Item::CulturePillar, arg),
        "culture_tradition" => data.verify_exists(Item::CultureTradition, arg),
        "decision" => data.verify_exists(Item::Decision, arg),
        "doctrine" => data.verify_exists(Item::Doctrine, arg),
        "dynasty" => data.verify_exists(Item::Dynasty, arg),
        "event_id" => data.verify_exists(Item::Event, arg),
        "faith" => data.verify_exists(Item::Faith, arg),
        "government_type" => data.verify_exists(Item::GovernmentType, arg),
        "house" => data.verify_exists(Item::House, arg),
        "mandate_type_qualification" => data.verify_exists(Item::DiarchyMandate, arg),
        "province" => data.verify_exists(Item::Province, arg),
        "religion" => data.verify_exists(Item::Religion, arg),
        "special_guest" => data.verify_exists(Item::SpecialGuest, arg),
        "struggle" => data.verify_exists(Item::Struggle, arg),
        "title" => data.verify_exists(Item::Title, arg),
        "trait" => data.verify_exists(Item::Trait, arg),
        "vassal_contract" | "vassal_contract_obligation_level" => {
            data.verify_exists(Item::VassalContract, arg);
        }
        &_ => (),
    }
}

pub fn validate_prefix_reference_token(token: &Token, data: &Everything, wanted: &str) {
    if let Some((prefix, arg)) = token.split_once(':') {
        validate_prefix_reference(&prefix, &arg, data);
        if prefix.is(wanted) {
            return;
        }
    }
    let msg = format!("should start with `{wanted}:` here");
    error(token, ErrorKey::Validation, &msg);
}

/// Check some iterator fields *before* the list scope has opened.
pub fn precheck_iterator_fields(
    ltype: ListType,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    match ltype {
        ListType::Any => {
            if let Some(bv) = block.get_field("percent") {
                if let Some(token) = bv.get_value() {
                    if let Some(num) = token.get_number() {
                        token.check_number();
                        if num > 1.0 {
                            let msg = "'percent' here needs to be between 0 and 1";
                            warn(token, ErrorKey::Range, msg);
                        }
                    }
                }
                validate_scriptvalue(bv, data, sc);
            }
            if let Some(bv) = block.get_field("count") {
                match bv {
                    BV::Value(token) if token.is("all") => (),
                    bv => validate_scriptvalue(bv, data, sc),
                }
            };
        }
        ListType::Ordered => {
            for field in &["position", "min", "max"] {
                if let Some(bv) = block.get_field(field) {
                    validate_scriptvalue(bv, data, sc);
                }
            }
        }
        ListType::Random | ListType::Every | ListType::None => (),
    }
}

/// This checks the fields that are only used in iterators.
/// It does not check "limit" because that is shared with the if/else blocks.
/// Returns true iff the iterator took care of its own tooltips
pub fn validate_iterator_fields(
    caller: &str,
    list_type: ListType,
    _data: &Everything,
    sc: &mut ScopeContext,
    vd: &mut Validator,
    tooltipped: &mut Tooltipped,
) {
    // undocumented
    if list_type == ListType::None {
        vd.ban_field("custom", || "lists");
    } else if vd.field_item("custom", Item::Localization) {
        *tooltipped = Tooltipped::No;
    }

    // undocumented
    if list_type != ListType::None && list_type != ListType::Any {
        vd.field_validated_blocks("alternative_limit", |b, data| {
            validate_normal_trigger(b, data, sc, *tooltipped);
        });
    } else {
        vd.ban_field("alternative_limit", || {
            "`every_`, `ordered_`, and `random_` lists"
        });
    }

    if list_type == ListType::Any {
        vd.field_any_cmp("percent"); // prechecked
        vd.field_any_cmp("count"); // prechecked
    } else {
        vd.ban_field("percent", || "`any_` lists");
        if caller != "while" {
            vd.ban_field("count", || "`while` and `any_` lists");
        }
    }

    if list_type == ListType::Ordered {
        vd.field_script_value("order_by", sc);
        vd.field("position"); // prechecked
        vd.field("min"); // prechecked
        vd.field("max"); // prechecked
        vd.field_bool("check_range_bounds");
    } else {
        vd.ban_field("order_by", || "`ordered_` lists");
        vd.ban_field("position", || "`ordered_` lists");
        if caller != "random_list" && caller != "duel" {
            vd.ban_field("min", || "`ordered_` lists, `random_list`, and `duel`");
            vd.ban_field("max", || "`ordered_` lists, `random_list`, and `duel`");
        }
        vd.ban_field("check_range_bounds", || "`ordered_` lists");
    }

    if list_type == ListType::Random {
        vd.field_validated_block_sc("weight", sc, validate_modifiers_with_base);
    } else {
        vd.ban_field("weight", || "`random_` lists");
    }
}

/// This checks the special fields for certain iterators, like `type =` in `every_relation`.
/// It doesn't check the generic ones like `limit` or the ordering ones for `ordered_*`.
pub fn validate_inside_iterator(
    name: &str,
    listtype: ListType,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    vd: &mut Validator,
    tooltipped: Tooltipped,
) {
    // Docs say that all three can take either list or variable, but global and local lists must be variable lists.
    if name == "in_list" {
        vd.req_field_one_of(&["list", "variable"]);
        if let Some(token) = vd.field_value("list") {
            sc.define_or_expect_list(token);
            sc.replace_list_entry(token.as_str(), token);
        }
    } else if name == "in_local_list" || name == "in_global_list" {
        vd.req_field("variable");
        vd.ban_field("list", || format!("{listtype}_in_list"));
    } else {
        vd.ban_field("list", || format!("{listtype}_in_list"));
        vd.ban_field("variable", || {
            format!(
                "`{listtype}_in_list`, `{listtype}_in_local_list`, or `{listtype}_in_global_list`"
            )
        });
    }

    if name == "in_de_facto_hierarchy" || name == "in_de_jure_hierarchy" {
        vd.field_validated_block("filter", |block, data| {
            validate_normal_trigger(block, data, sc, tooltipped);
        });
        vd.field_validated_block("continue", |block, data| {
            validate_normal_trigger(block, data, sc, tooltipped);
        });
    } else {
        let only_for =
            || format!("`{listtype}_in_de_facto_hierarchy` or `{listtype}_in_de_jure_hierarchy`");
        vd.ban_field("filter", only_for);
        vd.ban_field("continue", only_for);
    }

    if name == "county_in_region" {
        vd.req_field("region");
        vd.field_item("region", Item::Region);
    } else {
        vd.ban_field("region", || format!("`{listtype}_county_in_region`"));
    }

    if name == "court_position_holder" {
        vd.field_item("type", Item::CourtPosition);
    } else if name == "relation" {
        vd.req_field("type"); // without a type, it won't match any relationship
        for t in vd.field_values("type") {
            data.verify_exists(Item::Relation, t);
        }
    } else {
        vd.ban_field("type", || {
            format!("`{listtype}_court_position_holder` or `{listtype}_relation`")
        });
    }

    if name == "claim" {
        vd.field_choice("explicit", &["yes", "no", "all"]);
        vd.field_choice("pressed", &["yes", "no", "all"]);
    } else {
        vd.ban_field("explicit", || format!("`{listtype}_claim`"));
        vd.ban_field("pressed", || format!("`{listtype}_claim`"));
    }

    if name == "pool_character" {
        vd.req_field("province");
        if let Some(token) = block.get_field_value("province") {
            validate_target_ok_this(token, data, sc, Scopes::Province);
        }
    } else {
        vd.ban_field("province", || format!("`{listtype}_pool_character`"));
    }

    if sc.can_be(Scopes::Character) {
        vd.field_bool("only_if_dead");
        vd.field_bool("even_if_dead");
    } else {
        vd.ban_field("only_if_dead", || "lists of characters");
        vd.ban_field("even_if_dead", || "lists of characters");
    }

    if name == "character_struggle" {
        vd.field_choice("involvement", &["involved", "interloper"]);
    } else {
        vd.ban_field("involvement", || format!("`{listtype}_character_struggle`"));
    }

    if name == "connected_county" {
        // Undocumented
        vd.field_bool("invert");
        vd.field_numeric("max_naval_distance");
        vd.field_bool("allow_one_county_land_gap");
    } else {
        let only_for = || format!("`{listtype}_connected_county`");
        vd.ban_field("invert", only_for);
        vd.ban_field("max_naval_distance", only_for);
        vd.ban_field("allow_one_county_land_gap", only_for);
    }

    if name == "activity_phase_location"
        || name == "activity_phase_location_future"
        || name == "activity_phase_location_past"
    {
        vd.field_bool("unique");
    } else {
        let only_for = || format!("the `{listtype}_activity_phase_location` family of iterators");
        vd.ban_field("unique", only_for);
    }

    if name == "guest_subset" || name == "guest_subset_current_phase" {
        vd.field_item("name", Item::GuestSubset);
    } else {
        vd.ban_field("name", || {
            format!("`{listtype}_guest_subset` and `{listtype}_guest_subset_current_phase`")
        });
    }
    if name == "guest_subset" {
        vd.field_value("phase"); // TODO
    } else {
        vd.ban_field("phase", || format!("`{listtype}_guest_subset`"));
    }

    if name == "trait_in_category" {
        vd.field_value("category"); // TODO
    } else {
        // Don't ban, because it's a valid trigger
    }
}

pub fn validate_cost(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_script_value("gold", sc);
    vd.field_script_value("prestige", sc);
    vd.field_script_value("piety", sc);
    vd.field_bool("round");
}

pub fn validate_cost_with_renown(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_script_value("gold", sc);
    vd.field_script_value("prestige", sc);
    vd.field_script_value("piety", sc);
    vd.field_script_value("renown", sc);
    vd.field_bool("round");
}

pub fn validate_traits(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("virtues", validate_virtues_sins);
    vd.field_validated_block("sins", validate_virtues_sins);
}

pub fn validate_virtues_sins(block: &Block, data: &Everything) {
    // Can be single tokens ("wrathful") or assignments ("wrathful = 3")
    // or even wrathful = { scale = 2 weight = 2 } whatever that means
    let mut vd = Validator::new(block, data);
    for token in vd.values() {
        data.verify_exists(Item::Trait, token);
    }
    for (key, value) in vd.unknown_value_fields() {
        data.verify_exists(Item::Trait, key);
        value.expect_number();
    }
    for (key, block) in vd.unknown_block_fields() {
        data.verify_exists(Item::Trait, key);
        let mut vd = Validator::new(block, data);
        vd.field_numeric("scale");
        vd.field_numeric("weight");
    }
}

pub fn validate_compare_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    // `value` and `factor` are evaluated in the scope created by `target`
    sc.open_builder();
    let mut valid_target = false;
    vd.field_validated_value("target", |_, token, data| {
        valid_target = validate_scope_chain(token, data, sc, false);
    });
    sc.finalize_builder();
    if valid_target {
        vd.field_script_value("value", sc);
        vd.field_script_value("factor", sc);
    } else {
        vd.field("value");
        vd.field("factor");
    }
    sc.close();

    vd.fields_script_value("multiplier", sc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    vd.field_script_value("step", sc); // What does this do?
    vd.field_script_value("offset", sc); // What does this do?
    vd.field_validated_sc("desc", sc, validate_desc);
    vd.field_validated_block("trigger", |block, data| {
        validate_normal_trigger(block, data, sc, Tooltipped::No);
    });
}

pub fn validate_opinion_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    if let Some(target) = vd.field_value("who") {
        validate_target_ok_this(target, data, sc, Scopes::Character);
    }
    vd.req_field("opinion_target");
    if let Some(target) = vd.field_value("opinion_target") {
        validate_target_ok_this(target, data, sc, Scopes::Character);
    }
    vd.field_script_value("multiplier", sc);
    vd.field_validated_sc("desc", sc, validate_desc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    vd.field_script_value("step", sc); // What does this do?
    vd.field_validated_block("trigger", |block, data| {
        validate_normal_trigger(block, data, sc, Tooltipped::No);
    });
}

pub fn validate_ai_value_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    if let Some(target) = vd.field_value("who") {
        validate_target_ok_this(target, data, sc, Scopes::Character);
    }
    // TODO: verify that this actually works. It's only used 1 time in vanilla.
    vd.field_validated_block("dread_modified_ai_boldness", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_field("dreaded_character");
        vd.req_field("value");
        vd.field_target_ok_this("dreaded_character", sc, Scopes::Character);
        vd.field_script_value("value", sc);
    });
    vd.field_script_value("ai_boldness", sc);
    vd.field_script_value("ai_compassion", sc);
    vd.field_script_value("ai_energy", sc);
    vd.field_script_value("ai_greed", sc);
    vd.field_script_value("ai_honor", sc);
    vd.field_script_value("ai_rationality", sc);
    vd.field_script_value("ai_sociability", sc);
    vd.field_script_value("ai_vengefulness", sc);
    vd.field_script_value("ai_zeal", sc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    vd.field_validated_block("trigger", |block, data| {
        validate_normal_trigger(block, data, sc, Tooltipped::No);
    });
}

pub fn validate_compatibility_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    if let Some(target) = vd.field_value("who") {
        validate_target_ok_this(target, data, sc, Scopes::Character);
    }
    if let Some(target) = vd.field_value("compatibility_target") {
        validate_target_ok_this(target, data, sc, Scopes::Character);
    }
    vd.field_script_value("multiplier", sc);
    //vd.field_validated_sc("desc", sc, validate_desc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    vd.field_validated_block("trigger", |block, data| {
        validate_normal_trigger(block, data, sc, Tooltipped::No);
    });
}

pub fn validate_activity_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_target("object", sc, Scopes::Activity);
    vd.field_target("target", sc, Scopes::Character);
}

pub fn validate_scheme_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_target("object", sc, Scopes::Scheme);
    vd.field_target("target", sc, Scopes::Character);
}

pub fn validate_modifiers_with_base(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_validated("base", validate_non_dynamic_scriptvalue);
    vd.field_script_value("add", sc);
    vd.field_script_value("factor", sc);
    validate_modifiers(&mut vd, sc);
    validate_scripted_modifier_calls(vd, data, sc);
}

pub fn validate_modifiers(vd: &mut Validator, sc: &mut ScopeContext) {
    vd.field_validated_blocks("first_valid", |b, data| {
        let mut vd = Validator::new(b, data);
        validate_modifiers(&mut vd, sc);
    });
    vd.field_validated_blocks("modifier", |b, data| {
        validate_trigger("modifier", false, b, data, sc, Tooltipped::No, false);
    });
    vd.field_validated_blocks_sc("compare_modifier", sc, validate_compare_modifier);
    vd.field_validated_blocks_sc("opinion_modifier", sc, validate_opinion_modifier);
    vd.field_validated_blocks_sc("ai_value_modifier", sc, validate_ai_value_modifier);
    vd.field_validated_blocks_sc(
        "compatibility_modifier",
        sc,
        validate_compatibility_modifier,
    );

    // These are special single-use modifiers
    vd.field_validated_blocks_sc("scheme_modifier", sc, validate_scheme_modifier);
    vd.field_validated_blocks_sc("activity_modifier", sc, validate_activity_modifier);
}

pub fn validate_scripted_modifier_call(
    key: &Token,
    bv: &BV,
    modifier: &ScriptedModifier,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    match bv {
        BV::Value(token) => {
            if !modifier.macro_parms().is_empty() {
                error(token, ErrorKey::Macro, "expected macro arguments");
            } else if !token.is("yes") {
                warn(token, ErrorKey::Validation, "expected just modifier = yes");
            }
            modifier.validate_call(key, data, sc);
        }
        BV::Block(block) => {
            let parms = modifier.macro_parms();
            if parms.is_empty() {
                error_info(
                    block,
                    ErrorKey::Macro,
                    "modifier does not need macro arguments",
                    "you can just use it as modifier = yes",
                );
            } else {
                let mut vec = Vec::new();
                let mut vd = Validator::new(block, data);
                for parm in &parms {
                    vd.req_field(parm);
                    if let Some(token) = vd.field_value(parm) {
                        vec.push(token.clone());
                    } else {
                        return;
                    }
                }
                let args = parms.into_iter().zip(vec.into_iter()).collect();
                modifier.validate_macro_expansion(key, args, data, sc);
            }
        }
    }
}

pub fn validate_scripted_modifier_calls(
    mut vd: Validator,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    for (key, bv) in vd.unknown_fields() {
        if let Some(modifier) = data.scripted_modifiers.get(key.as_str()) {
            validate_scripted_modifier_call(key, bv, modifier, data, sc);
        } else {
            let msg = format!("unknown field `{key}`");
            warn(key, ErrorKey::UnknownField, &msg);
        }
    }
}

pub fn validate_ai_chance(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    match bv {
        BV::Value(t) => _ = t.expect_number(),
        BV::Block(b) => validate_modifiers_with_base(b, data, sc),
    }
}

/// Validate the left-hand part of a `target = { target_scope }` block.
/// The caller is expected to have done `sc.open_builder()` before calling and then do `sc.close()` after calling.
/// Returns true iff validation was complete.
/// `qeq` is true if the scope chain is to the left of a ?= operator.
pub fn validate_scope_chain(
    token: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
    qeq: bool,
) -> bool {
    let part_vec = token.split('.');
    for i in 0..part_vec.len() {
        let first = i == 0;
        let last = i + 1 == part_vec.len();
        let part = &part_vec[i];
        if let Some((prefix, arg)) = part.split_once(':') {
            if let Some((inscopes, outscope)) = scope_prefix(prefix.as_str()) {
                if inscopes == Scopes::None && !first {
                    let msg = format!("`{prefix}:` makes no sense except as first part");
                    warn(part, ErrorKey::Validation, &msg);
                }
                sc.expect(inscopes, &prefix);
                validate_prefix_reference(&prefix, &arg, data);
                if prefix.is("scope") {
                    if last && qeq {
                        sc.exists_scope(arg.as_str(), part);
                    }
                    sc.replace_named_scope(arg.as_str(), part);
                } else {
                    sc.replace(outscope, part.clone());
                }
            } else {
                let msg = format!("unknown prefix `{prefix}:`");
                error(part, ErrorKey::Validation, &msg);
                return false;
            }
        } else if part.lowercase_is("root")
            || part.lowercase_is("prev")
            || part.lowercase_is("this")
        {
            if !first {
                let msg = format!("`{part}` makes no sense except as first part");
                warn(part, ErrorKey::Validation, &msg);
            }
            if part.lowercase_is("root") {
                sc.replace_root();
            } else if part.lowercase_is("prev") {
                sc.replace_prev();
            } else {
                sc.replace_this();
            }
        } else if let Some((inscopes, outscope)) = scope_to_scope(part) {
            if inscopes == Scopes::None && !first {
                let msg = format!("`{part}` makes no sense except as first part");
                warn(part, ErrorKey::Validation, &msg);
            }
            sc.expect(inscopes, part);
            sc.replace(outscope, part.clone());
        } else {
            let msg = format!("unknown token `{part}`");
            error(part, ErrorKey::UnknownField, &msg);
            return false;
        }
    }
    true
}

pub fn validate_random_traits_list(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_script_value("count", sc);
    for (key, block) in vd.unknown_block_fields() {
        data.verify_exists(Item::Trait, key);
        let mut vd = Validator::new(block, data);
        vd.field_validated_block_sc("weight", sc, validate_modifiers_with_base);
        vd.field_validated_block("trigger", |block, data| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
    }
}

pub fn validate_random_culture(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    for (key, block) in vd.unknown_block_fields() {
        validate_target(key, data, sc, Scopes::Culture);
        let mut vd = Validator::new(block, data);
        vd.field_validated_block_sc("weight", sc, validate_modifiers_with_base);
        vd.field_validated_block("trigger", |block, data| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
    }
}

pub fn validate_random_faith(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    for (key, block) in vd.unknown_block_fields() {
        validate_target(key, data, sc, Scopes::Faith);
        let mut vd = Validator::new(block, data);
        vd.field_validated_block_sc("weight", sc, validate_modifiers_with_base);
        vd.field_validated_block("trigger", |block, data| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
    }
}

pub fn validate_maa_stats(vd: &mut Validator) {
    vd.field_numeric("pursuit");
    vd.field_numeric("screen");
    vd.field_numeric("damage");
    vd.field_numeric("toughness");
    vd.field_numeric("siege_value");
}

pub fn validate_ifelse_sequence(block: &Block, key_if: &str, key_elseif: &str, key_else: &str) {
    let mut seen_if = false;
    for (k, _, bv) in block.iter_items() {
        if let Some(key) = k {
            if key.is(key_if) {
                seen_if = true;
                continue;
            } else if key.is(key_elseif) {
                if !seen_if {
                    let msg = format!("`{key_elseif} without preceding `{key_if}`");
                    warn(key, ErrorKey::IfElse, &msg);
                }
                seen_if = true;
                continue;
            } else if key.is(key_else) {
                if !seen_if {
                    let msg = format!("`{key_else} without preceding `{key_if}`");
                    warn(key, ErrorKey::IfElse, &msg);
                }
                if let Some(block) = bv.get_block() {
                    if block.has_key("limit") {
                        // `else` with a `limit`, followed by another `else`, does work.
                        seen_if = true;
                        continue;
                    }
                }
            }
        }
        seen_if = false;
    }
}
