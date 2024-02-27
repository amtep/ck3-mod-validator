use crate::block::{Block, Comparator, Deserializer, Eq::*, Serializer, BV};
use crate::capnp::pdxfile_capnp::field;
use crate::report::{err, ErrorKey};
use crate::token::Token;

#[derive(Debug, Clone)]
pub struct Field(pub Token, pub Comparator, pub BV);

impl Field {
    pub fn into_key(self) -> Token {
        self.0
    }

    pub fn key(&self) -> &Token {
        &self.0
    }

    pub fn cmp(&self) -> Comparator {
        self.1
    }

    #[allow(dead_code)] // It's here for symmetry
    pub fn into_bv(self) -> BV {
        self.2
    }

    pub fn bv(&self) -> &BV {
        &self.2
    }

    pub fn is_eq(&self) -> bool {
        matches!(self.1, Comparator::Equals(Single))
    }

    pub fn is_eq_qeq(&self) -> bool {
        matches!(self.1, Comparator::Equals(Single | Question))
    }

    pub fn expect_eq(&self) -> bool {
        let Self(key, cmp, _) = self;
        if matches!(cmp, Comparator::Equals(Single)) {
            true
        } else {
            let msg = &format!("expected `{key} =`, found `{cmp}`");
            err(ErrorKey::Validation).msg(msg).loc(self).push();
            false
        }
    }

    pub fn describe(&self) -> &'static str {
        if self.is_eq_qeq() {
            match self.2 {
                BV::Value(_) => "assignment",
                BV::Block(_) => "definition",
            }
        } else {
            "comparison"
        }
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2.equivalent(&other.2)
    }

    pub fn expect_definition(&self) -> Option<(&Token, &Block)> {
        #[allow(clippy::single_match_else)] // too complicated for a `let`
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                return Some((key, block))
            }
            _ => {
                let msg = format!("expected definition, found {}", self.describe());
                err(ErrorKey::Structure).msg(msg).loc(self).push();
            }
        }
        None
    }

    pub fn expect_into_definition(self) -> Option<(Token, Block)> {
        #[allow(clippy::single_match_else)] // too complicated for a `let`
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                return Some((key, block))
            }
            _ => {
                let msg = format!("expected definition, found {}", self.describe());
                err(ErrorKey::Structure).msg(msg).loc(self).push();
            }
        }
        None
    }

    pub fn get_definition(&self) -> Option<(&Token, &Block)> {
        #[allow(clippy::single_match_else)] // too complicated for a `let`
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                Some((key, block))
            }
            _ => None,
        }
    }

    #[allow(dead_code)] // It's here for symmetry
    pub fn into_definition(self) -> Option<(Token, Block)> {
        #[allow(clippy::single_match_else)] // too complicated for a `let`
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                Some((key, block))
            }
            _ => None,
        }
    }

    pub fn get_assignment(&self) -> Option<(&Token, &Token)> {
        #[allow(clippy::single_match_else)] // too complicated for a `let`
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Value(token)) => {
                Some((key, token))
            }
            _ => None,
        }
    }

    #[allow(dead_code)] // It's here for symmetry
    pub fn expect_assignment(&self) -> Option<(&Token, &Token)> {
        #[allow(clippy::single_match_else)] // too complicated for a `let`
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Value(token)) => {
                return Some((key, token))
            }
            _ => {
                let msg = format!("expected assignment, found {}", self.describe());
                err(ErrorKey::Structure).msg(msg).loc(self).push();
            }
        }
        None
    }

    pub fn serialize(&self, s: &mut Serializer, builder: &mut field::Builder) {
        s.add_token(&mut builder.reborrow().init_token(), &self.0);
        builder.set_comparator(self.1.into());
        match &self.2 {
            BV::Value(token) => s.add_token(&mut builder.reborrow().get_bv().init_value(), token),
            BV::Block(block) => block.serialize(s, &mut builder.reborrow().get_bv().init_block()),
        }
    }

    pub fn deserialize(d: &Deserializer, reader: field::Reader) -> Option<Field> {
        let token = d.get_token(reader.get_token().ok()?);
        let comparator = reader.get_comparator().ok()?.into();
        let bv = match reader.get_bv().which().ok()? {
            field::bv::Value(token_reader) => BV::Value(d.get_token(token_reader.ok()?)),
            field::bv::Block(block_reader) => BV::Block(Block::deserialize(d, block_reader.ok()?)?),
        };
        Some(Field(token, comparator, bv))
    }
}
