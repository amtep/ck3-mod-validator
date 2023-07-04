use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{warn, ErrorKey};
use crate::token::Token;
use crate::validate::validate_color;

#[derive(Clone, Debug)]
pub struct NamedColor {}

impl NamedColor {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("colors") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::NamedColor, key, block, Box::new(Self {}));
            }
        } else {
            warn(key, ErrorKey::ParseError, "unexpected field");
        }
    }
}

impl DbKind for NamedColor {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        validate_color(block, data);
    }
}
