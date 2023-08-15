use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Holding {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Holding, Holding::add)
}

impl Holding {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        for token in block.get_field_values("flag") {
            db.add_flag(Item::HoldingFlag, token.clone());
        }
        db.add(Item::Holding, key, block, Box::new(Self {}));
    }
}

impl DbKind for Holding {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let modif = format!("{key}_build_speed");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_build_gold_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_build_piety_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_build_prestige_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_holding_build_speed");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_holding_build_gold_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_holding_build_piety_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_holding_build_prestige_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        let mut vd = Validator::new(block, data);
        vd.multi_field_value("flag");
        vd.field_item("primary_building", Item::Building);
        vd.field_list_items("buildings", Item::Building);
        vd.field_bool("can_be_inherited");
    }
}
