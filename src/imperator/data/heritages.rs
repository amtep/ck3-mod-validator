use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::context::ScopeContext;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;

#[derive(Clone, Debug)]
pub struct Heritage {}

impl Heritage {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Heritage, key, block, Box::new(Self {}));
    }
}

impl DbKind for Heritage {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
        vd.field_validated_block("trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
    }
}