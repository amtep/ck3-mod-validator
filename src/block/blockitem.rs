use crate::block::{Block, Comparator, Deserializer, Eq::*, Field, Serializer, BV};
use crate::capnp::pdxfile_capnp::block_item;
use crate::report::{err, ErrorKey};
use crate::token::Token;

#[derive(Debug, Clone)]
pub enum BlockItem {
    Value(Token),
    Block(Block),
    Field(Field),
}

impl BlockItem {
    pub fn expect_field(&self) -> Option<&Field> {
        if let BlockItem::Field(field) = self {
            Some(field)
        } else {
            let msg = format!("unexpected {}", self.describe());
            err(ErrorKey::Structure).msg(msg).info("Did you forget an = ?").loc(self).push();
            None
        }
    }

    pub fn expect_into_field(self) -> Option<Field> {
        if let BlockItem::Field(field) = self {
            Some(field)
        } else {
            let msg = format!("unexpected {}", self.describe());
            err(ErrorKey::Structure).msg(msg).info("Did you forget an = ?").loc(self).push();
            None
        }
    }

    pub fn get_field(&self) -> Option<&Field> {
        if let BlockItem::Field(field) = self {
            Some(field)
        } else {
            None
        }
    }

    pub fn is_field(&self) -> bool {
        matches!(self, BlockItem::Field(_))
    }

    pub fn get_value(&self) -> Option<&Token> {
        if let BlockItem::Value(token) = self {
            Some(token)
        } else {
            None
        }
    }

    pub fn expect_value(&self) -> Option<&Token> {
        if let BlockItem::Value(token) = self {
            Some(token)
        } else {
            let msg = format!("expected value, found {}", self.describe());
            err(ErrorKey::Structure).msg(msg).loc(self).push();
            None
        }
    }

    pub fn expect_into_value(self) -> Option<Token> {
        if let BlockItem::Value(token) = self {
            Some(token)
        } else {
            let msg = format!("expected value, found {}", self.describe());
            err(ErrorKey::Structure).msg(msg).loc(self).push();
            None
        }
    }

    #[allow(dead_code)] // It's here for symmetry
    pub fn get_block(&self) -> Option<&Block> {
        if let BlockItem::Block(block) = self {
            Some(block)
        } else {
            None
        }
    }

    pub fn expect_block(&self) -> Option<&Block> {
        if let BlockItem::Block(block) = self {
            Some(block)
        } else {
            let msg = format!("expected block, found {}", self.describe());
            err(ErrorKey::Structure).msg(msg).loc(self).push();
            None
        }
    }

    pub fn expect_into_block(self) -> Option<Block> {
        if let BlockItem::Block(block) = self {
            Some(block)
        } else {
            let msg = format!("expected block, found {}", self.describe());
            err(ErrorKey::Structure).msg(msg).loc(self).push();
            None
        }
    }

    pub fn get_definition(&self) -> Option<(&Token, &Block)> {
        if let Some(field) = self.get_field() {
            field.get_definition()
        } else {
            None
        }
    }

    pub fn expect_into_definition(self) -> Option<(Token, Block)> {
        if let Some(field) = self.expect_into_field() {
            field.expect_into_definition()
        } else {
            None
        }
    }

    pub fn expect_definition(&self) -> Option<(&Token, &Block)> {
        if let Some(field) = self.expect_field() {
            field.expect_definition()
        } else {
            None
        }
    }

    pub fn expect_assignment(&self) -> Option<(&Token, &Token)> {
        if let Some(field) = self.expect_field() {
            #[allow(clippy::single_match_else)] // too complicated for a `let`
            match field {
                Field(key, Comparator::Equals(Single | Question), BV::Value(token)) => {
                    return Some((key, token))
                }
                _ => {
                    let msg = format!("expected assignment, found {}", field.describe());
                    err(ErrorKey::Structure).msg(msg).loc(self).push();
                }
            }
        }
        None
    }

    pub fn get_assignment(&self) -> Option<(&Token, &Token)> {
        #[allow(clippy::single_match_else)] // too complicated for a `let`
        match self {
            BlockItem::Field(Field(
                key,
                Comparator::Equals(Single | Question),
                BV::Value(token),
            )) => Some((key, token)),
            _ => None,
        }
    }

    pub fn describe(&self) -> &'static str {
        match self {
            BlockItem::Value(_) => "value",
            BlockItem::Block(_) => "block",
            BlockItem::Field(field) => field.describe(),
        }
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        match self {
            BlockItem::Value(token) => {
                if let BlockItem::Value(t) = other {
                    token == t
                } else {
                    false
                }
            }
            BlockItem::Block(block) => {
                if let BlockItem::Block(b) = other {
                    block.equivalent(b)
                } else {
                    false
                }
            }
            BlockItem::Field(field) => {
                if let BlockItem::Field(f) = other {
                    field.equivalent(f)
                } else {
                    false
                }
            }
        }
    }

    pub fn serialize(&self, s: &mut Serializer, builder: &mut block_item::Builder) {
        match self {
            BlockItem::Value(token) => s.add_token(&mut builder.reborrow().init_value(), token),
            BlockItem::Block(block) => block.serialize(s, &mut builder.reborrow().init_block()),
            BlockItem::Field(field) => field.serialize(s, &mut builder.reborrow().init_field()),
        }
    }

    pub fn deserialize(d: &Deserializer, reader: block_item::Reader) -> Option<BlockItem> {
        match reader.which().ok()? {
            block_item::Value(token_reader) => {
                Some(BlockItem::Value(d.get_token(token_reader.ok()?)))
            }
            block_item::Block(block_reader) => {
                Some(BlockItem::Block(Block::deserialize(d, block_reader.ok()?)?))
            }
            block_item::Field(field_reader) => {
                Some(BlockItem::Field(Field::deserialize(d, field_reader.ok()?)?))
            }
        }
    }
}
