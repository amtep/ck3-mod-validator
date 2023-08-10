use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Decision {}

impl Decision {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Decision, key, block, Box::new(Self {}));
    }
}

impl DbKind for Decision {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_block("potential", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("allow", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("effect", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}
