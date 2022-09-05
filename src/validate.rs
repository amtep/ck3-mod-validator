/// A module for validation functions that are useful for more than one data module.
use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue};
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::everything::Everything;
use crate::token::Token;

pub fn validate_theme_background(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_block("trigger");
    // TODO: verify the background is defined
    vd.field_value("event_background");
    // TODO: check if `reference` actually works or is a mistake in vanilla
    vd.field_value("reference");
    vd.warn_remaining();
}

pub fn validate_theme_icon(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_block("trigger");
    // TODO: verify the file exists
    vd.field_value("reference"); // file
    vd.warn_remaining();
}

pub fn validate_theme_sound(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_block("trigger");
    vd.field_value("reference"); // event:/ resource reference
    vd.warn_remaining();
}

pub fn validate_cooldown(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    let mut count = 0;
    count += isize::from(vd.field_integer("years"));
    count += isize::from(vd.field_integer("months"));
    count += isize::from(vd.field_integer("days"));
    if count != 1 {
        warn(
            block,
            ErrorKey::Validation,
            "cooldown must have one of `years`, `months`, or `days`",
        );
    }
    vd.warn_remaining();
}

pub fn validate_color(block: &Block, _data: &Everything) {
    let mut count = 0;
    for (k, _, v) in block.iter_items() {
        if let Some(key) = k {
            error(key, ErrorKey::Validation, "expected color value");
        } else {
            match v {
                BlockOrValue::Token(t) => {
                    if let Ok(i) = t.as_str().parse::<isize>() {
                        if !(0..=255).contains(&i) {
                            error(
                                t,
                                ErrorKey::Validation,
                                "color values should be between 0 and 255",
                            );
                        }
                    } else if let Ok(f) = t.as_str().parse::<f64>() {
                        if !(0.0..=1.0).contains(&f) {
                            error(
                                t,
                                ErrorKey::Validation,
                                "color values should be between 0.0 and 1.0",
                            );
                        }
                    } else {
                        error(t, ErrorKey::Validation, "expected color value");
                    }
                    count += 1;
                }
                BlockOrValue::Block(b) => {
                    error(b, ErrorKey::Validation, "expected color value");
                }
            }
        }
    }
    if count != 3 {
        error(block, ErrorKey::Validation, "expected 3 color values");
    }
}

pub fn validate_scope_reference(prefix: &Token, arg: &Token, data: &Everything) {
    // TODO there are more to match
    match prefix.as_str() {
        "character" => data.characters.verify_exists(arg),
        "dynasty" => data.dynasties.verify_exists(arg),
        "faith" => data.religions.verify_faith_exists(arg),
        "house" => data.houses.verify_exists(arg),
        "province" => data.provinces.verify_exists(arg),
        "religion" => data.religions.verify_religion_exists(arg),
        "title" => data.titles.verify_exists(arg),
        &_ => (),
    }
}
