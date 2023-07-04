use std::fs::{read, read_to_string};
use std::path::Path;

use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};

use crate::block::Block;
use crate::fileset::FileEntry;
use crate::parse::pdxfile::parse_pdx;
use crate::report::ErrorKey;
use crate::report::{advice_info, error_info, warn};

/// If a windows-1252 file mistakenly starts with a UTF-8 BOM, this is
/// what it will look like after decoding
const BOM_FROM_1252: &str = "\u{00ef}\u{00bb}\u{00bf}";

pub struct PdxFile;

impl PdxFile {
    fn read_utf8(entry: &FileEntry, fullpath: &Path) -> Option<String> {
        match read_to_string(fullpath) {
            Ok(contents) => Some(contents),
            Err(e) => {
                error_info(
                    entry,
                    ErrorKey::ReadError,
                    "could not read file",
                    &format!("{e:#}"),
                );
                None
            }
        }
    }

    fn read_1252(entry: &FileEntry, fullpath: &Path) -> Option<String> {
        let bytes = match read(fullpath) {
            Ok(bytes) => bytes,
            Err(e) => {
                error_info(
                    entry,
                    ErrorKey::ReadError,
                    "could not read file",
                    &format!("{e:#}"),
                );
                return None;
            }
        };
        WINDOWS_1252.decode(&bytes, DecoderTrap::Strict).ok()
    }

    pub fn read_no_bom(entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        let contents = Self::read_utf8(entry, fullpath)?;
        Some(parse_pdx(entry, &contents))
    }

    pub fn read(entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        let contents = Self::read_utf8(entry, fullpath)?;
        if let Some(bomless) = contents.strip_prefix('\u{feff}') {
            Some(parse_pdx(entry, bomless))
        } else {
            warn(
                entry,
                ErrorKey::Encoding,
                "file must start with a UTF-8 BOM",
            );
            Some(parse_pdx(entry, &contents))
        }
    }

    pub fn read_optional_bom(entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        let contents = Self::read_utf8(entry, fullpath)?;
        if let Some(bomless) = contents.strip_prefix('\u{feff}') {
            Some(parse_pdx(entry, bomless))
        } else {
            Some(parse_pdx(entry, &contents))
        }
    }

    pub fn read_cp1252(entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        let contents = Self::read_1252(entry, fullpath)?;

        if let Some(bomless) = contents.strip_prefix(BOM_FROM_1252) {
            advice_info(
                entry,
                ErrorKey::Encoding,
                "file should not start with a UTF-8 BOM",
                "This kind of file is expected to be in Windows-1252 encoding",
            );
            Some(parse_pdx(entry, bomless))
        } else {
            Some(parse_pdx(entry, &contents))
        }
    }
}
