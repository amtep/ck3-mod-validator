use std::path::{Path, PathBuf};

use fnv::FnvHashMap;

use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::{
    validate_ai_chance, validate_cost_with_renown, validate_duration, validate_modifiers_with_base,
};

#[derive(Clone, Debug, Default)]
pub struct Interactions {
    interactions: FnvHashMap<String, Interaction>,
}

impl Interactions {
    pub fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.interactions.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(&key, &other.key, "interaction");
            }
        }
        self.interactions
            .insert(key.to_string(), Interaction::new(key, block));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.interactions.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.interactions.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Interactions {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/character_interactions")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(mut block) = PdxFile::read(entry, fullpath) else { return; };
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Interaction {
    key: Token,
    block: Block,
}

impl Interaction {
    pub fn new(key: Token, block: Block) -> Self {
        Interaction { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        // You're expected to use scope:actor and scope:recipient instead of root
        let mut sc = ScopeContext::new(Scopes::None, &self.key);
        sc.define_name("actor", Scopes::Character, &self.key);
        sc.define_name("recipient", Scopes::Character, &self.key);
        sc.define_name("hook", Scopes::Bool, &self.key);
        // TODO: figure out when these are available
        sc.define_name("secondary_actor", Scopes::Character, &self.key);
        sc.define_name("secondary_recipient", Scopes::Character, &self.key);
        // TODO: figure out if there's a better way than exhaustively matching on "interface" and "special_interaction"
        if let Some(target_type) = self.block.get_field_value("target_type") {
            if target_type.is("artifact") {
                sc.define_name("target", Scopes::Artifact, target_type);
            } else if target_type.is("title") {
                sc.define_name("target", Scopes::LandedTitle, target_type);
                sc.define_name("landed_title", Scopes::LandedTitle, target_type);
            }
        } else if let Some(interface) = self.block.get_field_value("interface") {
            if interface.is("interfere_in_war") || interface.is("call_ally") {
                sc.define_name("target", Scopes::War, interface);
            } else if interface.is("blackmail") {
                sc.define_name("target", Scopes::Secret, interface);
            } else if interface.is("council_task_interaction") {
                sc.define_name("target", Scopes::CouncilTask, interface);
            } else if interface.is("create_claimant_faction_against") {
                sc.define_name("landed_title", Scopes::LandedTitle, interface);
            }
        } else if let Some(special) = self.block.get_field_value("special_interaction") {
            if special.is("invite_to_council_interaction") {
                sc.define_name("target", Scopes::CouncilTask, special);
            } else if special.is("end_war_attacker_victory_interaction")
                || special.is("end_war_attacker_defeat_interaction")
                || special.is("end_war_white_peace_interaction")
            {
                sc.define_name("war", Scopes::War, special);
            } else if special.is("remove_scheme_interaction")
                || special.is("invite_to_scheme_interaction")
            {
                sc.define_name("scheme", Scopes::Scheme, special);
            }
        }
        for block in self.block.get_field_blocks("send_option") {
            if let Some(token) = block.get_field_value("flag") {
                sc.define_name(token.as_str(), Scopes::Bool, token);
            }
        }

        // Validate this early, to update the saved scopes in `sc`
        // TODO: figure out when exactly `redirect` is run
        vd.field_validated_block("redirect", |b, data| {
            validate_normal_effect(b, data, &mut sc, Tooltipped::No);
        });

        // Let ai_set_target set scope:target if it wants
        vd.field_validated_block("ai_set_target", |b, data| {
            validate_normal_effect(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_blocks("ai_targets"); // TODO
        vd.field_block("ai_target_quick_trigger"); // TODO

        vd.field_numeric("interface_priority");
        vd.field_bool("common_interaction");
        vd.field_item("category", Item::InteractionCategory);

        if let Some(icon_path) =
            data.get_defined_string_warn(&self.key, "NGameIcons|CHARACTER_INTERACTION_ICON_PATH")
        {
            if let Some(name) = vd.field_value("icon") {
                let pathname = format!("{icon_path}/{name}.dds");
                data.fileset.verify_exists_implied(&pathname, name);
            } else {
                let pathname = format!("{icon_path}/{}.dds", &self.key);
                data.item_used(Item::File, &pathname);
            }
            if let Some(name) = vd.field_value("alert_icon") {
                let pathname = format!("{icon_path}/{name}.dds");
                data.fileset.verify_exists_implied(&pathname, name);
            }
            if let Some(name) = vd.field_value("icon_small") {
                let pathname = format!("{icon_path}/{name}.dds");
                data.fileset.verify_exists_implied(&pathname, name);
            }
        } else {
            vd.field_value("icon");
            vd.field_value("alert_icon");
            vd.field_value("icon_small");
        }

        vd.field_validated_block("is_highlighted", |b, data| {
            validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::No);
        });
        vd.field_item("highlighted_reason", Item::Localization);

        vd.field_value("special_interaction");
        vd.field_value("special_ai_interaction");

        vd.field_bool("ai_intermediary_maybe");
        vd.field_bool("ai_maybe");
        vd.field_integer("ai_min_reply_days");
        vd.field_integer("ai_max_reply_days");

        vd.field_value("interface"); // TODO
        vd.field_item("scheme", Item::Scheme);
        vd.field_bool("popup_on_receive");
        vd.field_bool("pause_on_receive");
        vd.field_bool("force_notification");
        vd.field_bool("ai_accept_negotiation");

        vd.field_bool("hidden");

        vd.field_validated_sc(
            "use_diplomatic_range",
            &mut sc.clone(),
            validate_bool_or_trigger,
        );
        vd.field_bool("can_send_despite_rejection");
        vd.field_bool("ignores_pending_interaction_block");

        // The cooldowns seem to be in actor scope
        vd.field_validated_block_rerooted("cooldown", &sc, Scopes::Character, validate_duration);
        vd.field_validated_block_rerooted(
            "cooldown_against_recipient",
            &sc,
            Scopes::Character,
            validate_duration,
        );
        // undocumented, but used in marriage interaction
        vd.field_validated_block_rerooted(
            "recipient_recieve_cooldown",
            &sc,
            Scopes::Character,
            validate_duration,
        );
        vd.field_validated_block_rerooted(
            "category_cooldown",
            &sc,
            Scopes::Character,
            validate_duration,
        );
        vd.field_validated_block_rerooted(
            "category_cooldown_against_recipient",
            &sc,
            Scopes::Character,
            validate_duration,
        );

        vd.field_validated_block_rerooted(
            "ignore_recipient_recieve_cooldown",
            &sc,
            Scopes::Character,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, Tooltipped::No);
            },
        );

        // TODO: The ai_ name check is a heuristic. It would be better to check if the
        // is_shown trigger requires scope:actor to be is_ai = yes. But that's a long way off.
        if !self.key.as_str().starts_with("ai_") {
            data.localization.verify_exists(&self.key);
        }
        vd.field_validated_value("extra_icon", |key, token, data| {
            data.fileset.verify_exists(token);
            let loca = format!("{}_extra_icon", self.key);
            data.localization.verify_exists_implied(&loca, key);
        });
        vd.field_validated_block("should_use_extra_icon", |b, data| {
            validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::No);
        });

        vd.field_validated_block("is_shown", |b, data| {
            validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::No);
        });

        vd.field_validated_block("is_valid", |b, data| {
            validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::Yes);
        });
        vd.field_validated_block("is_valid_showing_failures_only", |b, data| {
            validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::FailuresOnly);
        });

        vd.field_validated_block("has_valid_target_showing_failures_only", |b, data| {
            validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::FailuresOnly);
        });
        vd.field_validated_block("has_valid_target", |b, data| {
            validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::Yes);
        });

        vd.field_validated_block("can_send", |b, data| {
            validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::Yes);
        });
        vd.field_validated_block("can_be_blocked", |b, data| {
            validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::Yes);
        });

        vd.field_validated_key_block("populate_actor_list", |key, block, data| {
            // TODO: this loca check and the one for recipient_secondary have a lot of false positives in vanilla.
            // Not sure why.
            let loca = format!("actor_secondary_{}", self.key);
            data.verify_exists_implied(Item::Localization, &loca, key);
            validate_normal_effect(block, data, &mut sc.clone(), Tooltipped::No);
        });
        vd.field_validated_key_block("populate_recipient_list", |key, block, data| {
            let loca = format!("recipient_secondary_{}", self.key);
            data.verify_exists_implied(Item::Localization, &loca, key);
            validate_normal_effect(block, data, &mut sc.clone(), Tooltipped::No);
        });

        vd.field_block("localization_values"); // TODO

        vd.field_validated_blocks("send_option", |b, data| {
            let mut vd = Validator::new(b, data);
            vd.req_field("localization");
            vd.req_field("flag");
            vd.field_item("localization", Item::Localization);
            vd.field_value("flag");
            vd.field_validated_block("is_shown", |b, data| {
                validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::No);
            });
            vd.field_validated_block("is_valid", |b, data| {
                validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::FailuresOnly);
            });
            vd.field_validated_block("starts_enabled", |b, data| {
                validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::No);
            });
            vd.field_validated_block("can_be_changed", |b, data| {
                validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::No);
            });
            vd.field_validated_sc("current_description", &mut sc.clone(), validate_desc);
            vd.field_bool("can_invalidate_interaction");
        });

        vd.field_bool("send_options_exclusive");
        vd.field_validated_block("on_send", |b, data| {
            validate_normal_effect(b, data, &mut sc.clone(), Tooltipped::Yes);
        });
        vd.field_validated_block("on_accept", |b, data| {
            validate_normal_effect(b, data, &mut sc.clone(), Tooltipped::Yes);
        });
        vd.field_validated_block("on_decline", |b, data| {
            validate_normal_effect(b, data, &mut sc.clone(), Tooltipped::Yes);
        });
        vd.field_validated_block("on_blocked_effect", |b, data| {
            validate_normal_effect(b, data, &mut sc.clone(), Tooltipped::No);
        });
        vd.field_validated_block("pre_auto_accept", |b, data| {
            validate_normal_effect(b, data, &mut sc.clone(), Tooltipped::No);
        });
        vd.field_validated_block("on_auto_accept", |b, data| {
            validate_normal_effect(b, data, &mut sc.clone(), Tooltipped::Yes);
        });
        vd.field_validated_block("on_intermediary_accept", |b, data| {
            validate_normal_effect(b, data, &mut sc.clone(), Tooltipped::Yes);
        });
        vd.field_validated_block("on_intermediary_decline", |b, data| {
            validate_normal_effect(b, data, &mut sc.clone(), Tooltipped::Yes);
        });

        vd.field_integer("ai_frequency"); // months

        // This seems to be in character scope
        vd.field_validated_block_rerooted(
            "ai_potential",
            &sc,
            Scopes::Character,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, Tooltipped::Yes);
            },
        );
        if let Some(token) = self.block.get_key("ai_potential") {
            if self.block.get_field_integer("ai_frequency").unwrap_or(0) == 0
                && !self.key.is("revoke_title_interaction")
            {
                let msg = "`ai_potential` will not be used if `ai_frequency` is 0";
                warn(token, ErrorKey::Unneeded, msg);
            }
        }
        vd.field_validated_sc(
            "ai_intermediary_accept",
            &mut sc.clone(),
            validate_ai_chance,
        );

        // These seem to be in character scope
        vd.field_validated_block_rerooted(
            "ai_accept",
            &sc,
            Scopes::Character,
            validate_modifiers_with_base,
        );
        vd.field_validated_block_rerooted(
            "ai_will_do",
            &sc,
            Scopes::Character,
            validate_modifiers_with_base,
        );

        vd.field_validated_sc("desc", &mut sc.clone(), validate_desc);
        vd.field_choice("greeting", &["negative", "positive"]);
        vd.field_validated_sc("prompt", &mut sc.clone(), validate_desc);
        vd.field_validated_sc(
            "intermediary_notification_text",
            &mut sc.clone(),
            validate_desc,
        );
        vd.field_validated_sc("notification_text", &mut sc.clone(), validate_desc);
        vd.field_validated_sc("on_decline_summary", &mut sc.clone(), validate_desc);
        vd.field_item("answer_block_key", Item::Localization);
        vd.field_item("answer_accept_key", Item::Localization);
        vd.field_item("answer_reject_key", Item::Localization);
        vd.field_item("answer_acknowledge_key", Item::Localization);
        vd.field_item("options_heading", Item::Localization);
        vd.field_item("pre_answer_maybe_breakdown_key", Item::Localization);
        vd.field_item("pre_answer_no_breakdown_key", Item::Localization);
        vd.field_item("pre_answer_yes_breakdown_key", Item::Localization);
        vd.field_item("pre_answer_maybe_key", Item::Localization);
        vd.field_item("pre_answer_no_key", Item::Localization);
        vd.field_item("pre_answer_yes_key", Item::Localization);
        vd.field_item("intermediary_breakdown_maybe", Item::Localization);
        vd.field_item("intermediary_breakdown_no", Item::Localization);
        vd.field_item("intermediary_breakdown_yes", Item::Localization);
        vd.field_item("intermediary_answer_accept_key", Item::Localization);
        vd.field_item("intermediary_answer_reject_key", Item::Localization);
        vd.field_item("reply_item_key", Item::Localization);
        vd.field_item("send_name", Item::Localization);

        vd.field_bool("needs_recipient_to_open");
        vd.field_bool("show_effects_in_notification");
        vd.field_bool("diarch_interaction");
        vd.field_validated_sc("auto_accept", &mut sc.clone(), validate_bool_or_trigger);

        vd.field_choice("target_type", &["artifact", "title", "none"]);
        vd.field_value("target_filter"); // TODO

        // root is the character being picked
        vd.field_validated_block_rerooted(
            "can_be_picked",
            &sc,
            Scopes::Character,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, Tooltipped::Yes);
            },
        );
        vd.field_validated_block("can_be_picked_title", |b, data| {
            validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::Yes);
        });
        vd.field_validated_block("can_be_picked_artifact", |b, data| {
            validate_normal_trigger(b, data, &mut sc.clone(), Tooltipped::Yes);
        });

        // Experimentation showed that even the cost block has scope none
        vd.field_validated_block_rerooted("cost", &sc, Scopes::None, validate_cost_with_renown);
    }
}

fn validate_bool_or_trigger(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    match bv {
        BV::Value(t) => {
            if !t.is("yes") && !t.is("no") {
                warn(t, ErrorKey::Validation, "expected yes or no");
            }
        }
        BV::Block(b) => {
            validate_normal_trigger(b, data, sc, Tooltipped::No);
        }
    }
}
