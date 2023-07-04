use std::path::{Path, PathBuf};

use fnv::FnvHashMap;

use crate::block::BV;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Defines {
    defines: FnvHashMap<String, Define>,
}

impl Defines {
    pub fn load_item(&mut self, group: Token, name: Token, bv: &BV) {
        let key = format!("{}|{}", &group, &name);
        if let Some(other) = self.defines.get(&key) {
            if other.name.loc.kind >= name.loc.kind && !bv.equivalent(&other.bv) {
                dup_error(&name, &other.name, "define");
            }
        }
        self.defines
            .insert(key, Define::new(group, name, bv.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.defines.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.defines.values() {
            item.validate(data);
        }
    }

    pub fn get_string(&self, key: &str) -> Option<&Token> {
        self.defines.get(key).and_then(|d| d.bv.get_value())
    }
}

impl FileHandler for Defines {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/defines")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(mut block) = PdxFile::read(entry, fullpath) else { return; };
        for (group, block) in block.drain_definitions_warn() {
            for (name, bv) in block.iter_bv_definitions_warn() {
                self.load_item(group.clone(), name.clone(), bv);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Define {
    group: Token,
    name: Token,
    bv: BV,
}

impl Define {
    pub fn new(group: Token, name: Token, bv: BV) -> Self {
        Self { group, name, bv }
    }

    pub fn key(&self) -> String {
        format!("{}|{}", self.group, self.name)
    }

    #[allow(clippy::unused_self)]
    pub fn validate(&self, _data: &Everything) {
        // TODO: validate that each define is the right 'type',
        // such as a path, a number, or a block of numeric values
    }
}
