use fnv::FnvHashMap;

use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::helpers::dup_error;
use crate::item::Item;
use crate::report::{error, warn, warn2, ErrorKey};
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Gene {}

impl Gene {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        match key.as_str() {
            "color_genes" => {
                for (k, b) in block.drain_definitions_warn() {
                    ColorGene::add(db, k, b);
                }
            }
            "age_presets" => {
                for (k, b) in block.drain_definitions_warn() {
                    AgePresetGene::add(db, k, b);
                }
            }
            "decal_atlases" => {
                for (_k, _b) in block.drain_definitions_warn() {
                    // TODO: no examples in vanilla
                }
            }
            "morph_genes" => {
                for (k, b) in block.drain_definitions_warn() {
                    MorphGene::add(db, k, b, false);
                }
            }
            "accessory_genes" => {
                for (k, b) in block.drain_definitions_warn() {
                    AccessoryGene::add(db, k, b);
                }
            }
            "special_genes" => {
                for (k, mut b) in block.drain_definitions_warn() {
                    match k.as_str() {
                        "morph_genes" => {
                            for (k, b) in b.drain_definitions_warn() {
                                MorphGene::add(db, k, b, true);
                            }
                        }
                        "accessory_genes" => {
                            for (k, b) in b.drain_definitions_warn() {
                                AccessoryGene::add(db, k, b);
                            }
                        }
                        _ => warn(k, ErrorKey::ParseError, "unknown gene type"),
                    }
                }
            }
            _ => warn(key, ErrorKey::ParseError, "unknown gene type"),
        }
    }

    pub fn verify_has_template(category: &str, template: &Token, data: &Everything) {
        if !data.item_has_property(Item::GeneCategory, category, template.as_str()) {
            let msg = format!("gene {category} does not have template {template}");
            error(template, ErrorKey::MissingItem, &msg);
        }
    }
}

#[derive(Clone, Debug)]
pub struct ColorGene {}

impl ColorGene {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GeneCategory, key, block, Box::new(Self {}));
    }
}

impl DbKind for ColorGene {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        data.verify_exists(Item::Localization, key);

        vd.req_field("group");
        vd.req_field("color");
        vd.req_field("blend_range");

        vd.field_item("sync_inheritance_with", Item::GeneCategory);
        vd.field_value("group"); // TODO
        vd.field_value("color"); // TODO
        vd.field_validated_block("blend_range", validate_gene_range);
    }

    fn validate_use(
        &self,
        _key: &Token,
        _block: &Block,
        data: &Everything,
        _call_key: &Token,
        call_block: &Block,
    ) {
        let mut vd = Validator::new(call_block, data);
        vd.req_tokens_numbers_exactly(4);
    }
}

#[derive(Clone, Debug)]
pub struct AgePresetGene {}

impl AgePresetGene {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GeneAgePreset, key, block, Box::new(Self {}));
    }
}

impl DbKind for AgePresetGene {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        validate_age(block, data);
    }

    fn validate_use(
        &self,
        _key: &Token,
        _block: &Block,
        _data: &Everything,
        call_key: &Token,
        _call_block: &Block,
    ) {
        warn(
            call_key,
            ErrorKey::Validation,
            "cannot define age preset genes",
        );
    }
}

#[derive(Clone, Debug)]
pub struct MorphGene {
    special_gene: bool,
    templates: FnvHashMap<String, Token>,
}

impl MorphGene {
    pub fn add(db: &mut Db, key: Token, block: Block, special_gene: bool) {
        let mut templates = FnvHashMap::default();
        for (key, _block) in block.iter_definitions() {
            if key.is("ugliness_feature_categories") {
                continue;
            }
            if let Some(other) = templates.get(key.as_str()) {
                dup_error(key, other, "morph gene template");
            }
            templates.insert(key.to_string(), key.clone());
        }
        db.add(
            Item::GeneCategory,
            key,
            block,
            Box::new(Self {
                special_gene,
                templates,
            }),
        );
    }
}

impl DbKind for MorphGene {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_list("ugliness_feature_categories"); // TODO: options
        vd.field_bool("can_have_portrait_extremity_shift");
        // TODO value?
        if let Some(token) = vd.field_value("group") {
            if self.special_gene {
                let msg = "adding a group to a gene under special_genes will make the ruler designer crash";
                error(token, ErrorKey::Crash, msg);
            }
        }
        for (_key, block) in vd.unknown_block_fields() {
            validate_morph_gene(block, data);
        }
    }

    fn has_property(
        &self,
        _key: &Token,
        _block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        self.templates.contains_key(property)
    }

    fn validate_property_use(
        &self,
        _key: &Token,
        block: &Block,
        property: &Token,
        caller: &str,
        data: &Everything,
    ) {
        validate_portrait_modifier_use(block, data, property, caller);
    }

    fn validate_use(
        &self,
        _key: &Token,
        _block: &Block,
        data: &Everything,
        call_key: &Token,
        call_block: &Block,
    ) {
        let mut vd = Validator::new(call_block, data);
        let mut count = 0;
        for token in vd.values() {
            if count % 2 == 0 {
                if !token.is("") && !self.templates.contains_key(token.as_str()) {
                    let msg = format!("Gene template {token} not found in category {call_key}");
                    error(token, ErrorKey::MissingItem, &msg);
                }
            } else if let Some(i) = token.expect_integer() {
                if !(0..=256).contains(&i) {
                    warn(token, ErrorKey::Range, "expected value from 0 to 256");
                }
            }
            count += 1;
            if count > 4 {
                let msg = "too many values in this gene";
                error(token, ErrorKey::Validation, msg);
                break;
            }
        }
        if count < 4 {
            let msg = "too few values in this gene";
            error(call_block, ErrorKey::Validation, msg);
        }
    }
}

fn validate_portrait_modifier_use(
    block: &Block,
    data: &Everything,
    property: &Token,
    caller: &str,
) {
    // get template
    if let Some(block) = block.get_field_block(property.as_str()) {
        // loop over body types
        for field in &["male", "female", "boy", "girl"] {
            // get weighted settings
            if let Some(block) = block.get_field_block(field) {
                for (_, token) in block.iter_assignments() {
                    if token.is("empty") {
                        continue;
                    }
                    let loca = format!("PORTRAIT_MODIFIER_{caller}_{token}");
                    if !data.item_exists(Item::Localization, &loca) {
                        let msg = format!("missing localization key {loca}");
                        warn2(
                            property,
                            ErrorKey::MissingLocalization,
                            &msg,
                            token,
                            "this setting",
                        );
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AccessoryGene {
    templates: FnvHashMap<String, Token>,
}

impl AccessoryGene {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        let mut templates = FnvHashMap::default();
        for (key, block) in block.iter_definitions() {
            if key.is("ugliness_feature_categories") {
                continue;
            }
            if let Some(other) = templates.get(key.as_str()) {
                dup_error(key, other, "accessory gene template");
            }
            templates.insert(key.to_string(), key.clone());

            if let Some(tags) = block.get_field_value("set_tags") {
                for tag in tags.split(',') {
                    db.add_flag(Item::AccessoryTag, tag);
                }
            }
        }
        db.add(Item::GeneCategory, key, block, Box::new(Self { templates }));
    }

    pub fn has_template_setting(
        _key: &Token,
        block: &Block,
        _data: &Everything,
        template: &str,
        setting: &str,
    ) -> bool {
        if template == "ugliness_feature_categories" {
            return false;
        }
        if let Some(block) = block.get_field_block(template) {
            for field in &["male", "female", "boy", "girl"] {
                // get weighted settings
                if let Some(block) = block.get_field_block(field) {
                    for (_, token) in block.iter_assignments() {
                        if token.is("empty") {
                            continue;
                        }
                        if token.is(setting) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

impl DbKind for AccessoryGene {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_bool("inheritable");
        vd.field_value("group");
        for (_key, block) in vd.unknown_block_fields() {
            validate_accessory_gene(block, data);
        }
    }

    fn has_property(
        &self,
        _key: &Token,
        _block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        self.templates.contains_key(property)
    }

    fn validate_property_use(
        &self,
        _key: &Token,
        block: &Block,
        property: &Token,
        caller: &str,
        data: &Everything,
    ) {
        validate_portrait_modifier_use(block, data, property, caller);
    }

    fn validate_use(
        &self,
        _key: &Token,
        _block: &Block,
        data: &Everything,
        call_key: &Token,
        call_block: &Block,
    ) {
        let mut vd = Validator::new(call_block, data);
        let mut count = 0;
        for token in vd.values() {
            if count % 2 == 0 {
                if !token.is("") && !self.templates.contains_key(token.as_str()) {
                    let msg = format!("Gene template {token} not found in category {call_key}");
                    error(token, ErrorKey::MissingItem, &msg);
                }
            } else if let Some(i) = token.expect_integer() {
                if !(0..=256).contains(&i) {
                    warn(token, ErrorKey::Range, "expected value from 0 to 256");
                }
            }
            count += 1;
            if count > 4 {
                let msg = "too many values in this gene";
                error(token, ErrorKey::Validation, msg);
                break;
            }
        }
        if count < 4 {
            let msg = "too few values in this gene";
            error(call_block, ErrorKey::Validation, msg);
        }
    }
}

fn validate_age_field(bv: &BV, data: &Everything) {
    match bv {
        BV::Value(token) => data.verify_exists(Item::GeneAgePreset, token),
        BV::Block(block) => validate_age(block, data),
    }
}

fn validate_age(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_field("mode");
    vd.req_field("curve");

    vd.field_value("mode"); // TODO
    vd.field_validated_block("curve", validate_curve);
}

fn validate_curve(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for block in vd.blocks() {
        validate_curve_range(block, data);
    }
}

fn validate_hsv_curve(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for block in vd.blocks() {
        validate_hsv_curve_range(block, data);
    }
}

fn validate_curve_range(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;
    for token in vd.values() {
        if let Some(v) = token.expect_number() {
            count += 1;
            #[allow(clippy::collapsible_else_if)]
            if count == 1 {
                if !(0.0..=1.0).contains(&v) {
                    error(token, ErrorKey::Range, "expected number from 0.0 to 1.0");
                }
            } else {
                if !(-1.0..=1.0).contains(&v) {
                    error(token, ErrorKey::Range, "expected number from -1.0 to 1.0");
                }
            }
        }
    }
    if count != 2 {
        error(block, ErrorKey::Validation, "expected exactly 2 numbers");
    }
}

fn validate_hsv_curve_range(block: &Block, data: &Everything) {
    let mut found_first = false;
    let mut found_second = false;

    for (k, _cmp, bv) in block.iter_items() {
        if let Some(key) = k {
            warn(key, ErrorKey::Validation, "unknown field");
        } else if !found_first {
            if let Some(token) = bv.expect_value() {
                if let Some(v) = token.expect_number() {
                    found_first = true;
                    if !(0.0..=1.0).contains(&v) {
                        error(token, ErrorKey::Range, "expected number from 0.0 to 1.0");
                    }
                }
            }
        } else if !found_second {
            if let Some(block) = bv.expect_block() {
                found_second = true;
                let mut count = 0;
                let mut vd = Validator::new(block, data);
                for token in vd.values() {
                    if let Some(v) = token.expect_number() {
                        count += 1;
                        if !(-1.0..=1.0).contains(&v) {
                            error(token, ErrorKey::Range, "expected number from -1.0 to 1.0");
                        }
                    }
                }
                if count != 3 {
                    error(block, ErrorKey::Validation, "expected exactly 3 numbers");
                }
            }
        }
    }
}

fn validate_gene_range(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;
    for token in vd.values() {
        if let Some(v) = token.expect_number() {
            count += 1;
            if !(0.0..=1.0).contains(&v) {
                error(token, ErrorKey::Range, "expected number from 0.0 to 1.0");
            }
        }
    }
    if count != 2 {
        error(block, ErrorKey::Validation, "expected exactly 2 numbers");
    }
}

fn validate_morph_gene(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_field("index");
    vd.field_integer("index"); // TODO: verify unique indices
    vd.field_bool("generic");
    vd.field_bool("visible");
    vd.field_value("positive_mirror"); // TODO
    vd.field_value("negative_mirror"); // TODO
    let choices = &["male", "female", "boy", "girl"];
    for field in choices {
        vd.field_validated(field, |bv, data| {
            match bv {
                BV::Value(token) => {
                    // TODO: if it refers to another field, check that following the chain of fields eventually reaches a block
                    if !choices.contains(&token.as_str()) {
                        let msg = format!("expected one of {}", choices.join(", "));
                        warn(token, ErrorKey::Choice, &msg);
                    }
                }
                BV::Block(block) => {
                    let mut vd = Validator::new(block, data);
                    vd.field_validated_blocks("setting", validate_gene_setting);
                    vd.field_validated_blocks("decal", validate_gene_decal);
                    vd.field_validated_blocks("texture_override", validate_texture_override);
                    vd.field_validated_block("hair_hsv_shift_curve", validate_shift_curve);
                    vd.field_validated_block("eye_hsv_shift_curve", validate_shift_curve);
                    vd.field_validated_block("skin_hsv_shift_curve", validate_shift_curve);
                }
            }
        });
    }
}

fn validate_accessory_gene(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_field("index");
    vd.field_integer("index"); // TODO: verify unique indices
    vd.field_value("set_tags");
    let choices = &["male", "female", "boy", "girl"];
    for field in choices {
        vd.field_validated(field, |bv, data| {
            match bv {
                BV::Value(token) => {
                    // TODO: if it refers to another field, check that following the chain of fields eventually reaches a block
                    if !choices.contains(&token.as_str()) {
                        let msg = format!("expected one of {}", choices.join(", "));
                        warn(token, ErrorKey::Choice, &msg);
                    }
                }
                BV::Block(block) => {
                    let mut vd = Validator::new(block, data);
                    for (_weight, token) in vd.integer_values() {
                        if !token.is("empty") {
                            data.verify_exists(Item::Accessory, token);
                        }
                    }
                }
            }
        });
    }
}

fn validate_gene_setting(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_field("attribute");
    vd.req_field_one_of(&["value", "curve"]);
    vd.field_item("attribute", Item::GeneAttribute);
    vd.field_validated_block("value", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_field("min");
        vd.req_field("max");
        vd.field_numeric("min");
        vd.field_numeric("max");
    });
    vd.field_validated_block("curve", validate_curve);
    vd.field_validated("age", validate_age_field);
    if let Some(token) = vd.field_value("required_tags") {
        for tag in token.split(',') {
            if tag.starts_with("not(") {
                let real_tag = &tag.split('(')[1].split(')')[0];
                data.verify_exists(Item::AccessoryTag, real_tag);
            } else {
                data.verify_exists(Item::AccessoryTag, &tag);
            }
        }
    }
}

fn validate_gene_decal(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_field("body_part");
    vd.req_field("textures");
    vd.req_field("priority");
    vd.field_value("body_part"); // TODO
    vd.field_validated_blocks("textures", validate_decal_textures);
    vd.field_validated_blocks("alpha_curve", validate_curve);
    vd.field_validated_blocks("blend_modes", validate_blend_modes);
    vd.field_integer("priority");
    vd.field_validated("age", validate_age_field);
    vd.field_choice("decal_apply_order", &["pre_skin_color", "post_skin_color"]);
}

fn validate_decal_textures(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    // TODO: validate that it's a dds? What properties should the dds have?
    vd.field_item("diffuse", Item::File);
    vd.field_item("normal", Item::File);
    vd.field_item("specular", Item::File);
    vd.field_item("properties", Item::File);
}

fn validate_texture_override(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_field("weight");
    vd.field_integer("weight");
    // TODO: validate that it's a dds? What properties should the dds have?
    vd.field_item("diffuse", Item::File);
    vd.field_item("normal", Item::File);
    vd.field_item("specular", Item::File);
    vd.field_item("properties", Item::File);
}

fn validate_blend_modes(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let choices = &["overlay", "replace", "hard_light", "multiply"];
    vd.field_choice("diffuse", choices);
    vd.field_choice("normal", choices);
    vd.field_choice("properties", choices);
}

fn validate_shift_curve(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_field("curve");
    vd.field_validated_block("curve", validate_hsv_curve);
    vd.field_validated("age", validate_age_field);
}
