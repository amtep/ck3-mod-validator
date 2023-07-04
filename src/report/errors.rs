use std::fs::read;
use std::io::{Stderr, Stdout, Write};
use std::path::PathBuf;

use encoding::all::{UTF_8, WINDOWS_1252};
use encoding::{DecoderTrap, Encoding};
use fnv::{FnvHashMap, FnvHashSet};

use crate::fileset::FileKind;
use crate::report::error_loc::ErrorLoc;
use crate::report::filter::ReportFilter;
use crate::report::writer::log_report;
use crate::report::{ErrorKey, RulesListType};
use crate::report::{Confidence, LogLevel, LogReport, OutputStyle, PointedMessage, Severity};
use crate::token::Loc;

static mut ERRORS: Option<Errors> = None;

type ErrorRecord = (Loc, ErrorKey, String, Option<Loc>, Option<Loc>);

#[derive(Default, Debug)]
pub struct Errors {
    /// The CK3 game directory
    vanilla_root: PathBuf,
    /// Extra CK3 directory loaded before `vanilla_root`
    clausewitz_root: PathBuf,
    /// Extra CK3 directory loaded before `vanilla_root`
    jomini_root: PathBuf,
    /// The mod directory
    mod_root: PathBuf,
    /// Extra loaded mods' directories
    loaded_mods: Vec<PathBuf>,
    /// Extra loaded mods' error tags
    pub(crate) loaded_mods_labels: Vec<String>,

    /// Errors that have already been logged (to avoid duplication, which is common
    /// when validating macro expanded triggers and effects)
    seen: FnvHashSet<ErrorRecord>,

    filecache: FnvHashMap<PathBuf, String>,

    /// Determines whether a report should be printed.
    pub(crate) filter: ReportFilter,
    /// Output color and style configuration.
    pub(crate) styles: OutputStyle,
    pub(crate) max_line_length: Option<usize>,
}

impl Errors {
    pub(crate) fn get_line(&mut self, loc: &Loc) -> Option<String> {
        if loc.line == 0 {
            return None;
        }
        let pathname = match loc.kind {
            FileKind::Internal => (*loc.pathname).clone(),
            FileKind::Clausewitz => self.clausewitz_root.join(&*loc.pathname),
            FileKind::Jomini => self.jomini_root.join(&*loc.pathname),
            FileKind::Vanilla => self.vanilla_root.join(&*loc.pathname),
            FileKind::LoadedMod(idx) => self.loaded_mods[idx as usize].join(&*loc.pathname),
            FileKind::Mod => self.mod_root.join(&*loc.pathname),
        };
        if let Some(contents) = self.filecache.get(&pathname) {
            return contents.lines().nth(loc.line - 1).map(str::to_string);
        }
        let bytes = read(&pathname).ok()?;
        let contents = match UTF_8.decode(&bytes, DecoderTrap::Strict) {
            Ok(contents) => contents,
            Err(_) => WINDOWS_1252.decode(&bytes, DecoderTrap::Strict).ok()?,
        };
        let line = contents.lines().nth(loc.line - 1).map(str::to_string);
        self.filecache.insert(pathname, contents);
        line
    }

    /// Perform some checks to see whether the report should actually be logged.
    /// If yes, it will do so.
    fn push_report(&mut self, report: &LogReport) {
        if !self.filter.should_print_report(report) {
            return;
        }
        let loc = report.primary().location.clone();
        let loc2 = report.pointers.get(1).map(|p| p.location.clone());
        let loc3 = report.pointers.get(2).map(|p| p.location.clone());
        let index = (loc, report.key, report.msg.to_string(), loc2, loc3);
        if self.seen.contains(&index) {
            return;
        }
        self.seen.insert(index);
        log_report(self, report);
    }

    pub fn log_abbreviated(&mut self, loc: &Loc, key: ErrorKey) {
        if loc.line == 0 {
            println!(
                "({key}) {}",
                loc.pathname.to_string_lossy()
            );
        } else if let Some(line) = self.get_line(loc) {
            println!("({key}) {line}");
        }
    }

    pub fn push_abbreviated<E: ErrorLoc>(&mut self, eloc: E, key: ErrorKey) {
        let loc = eloc.into_loc();
        let index = (loc.clone(), key, String::new(), None, None);
        if self.seen.contains(&index) {
            return;
        }
        self.seen.insert(index);
        self.log_abbreviated(&loc, key);
    }

    pub fn push_header(&mut self, _key: ErrorKey, msg: &str) {
        println!("{msg}");
    }

    pub fn get_mut() -> &'static mut Self {
        // Safe because we're single-threaded, and won't start reporting
        // validation errors until we're well past initialization.
        unsafe {
            if ERRORS.is_none() {
                ERRORS = Some(Errors::default());
            }
            match ERRORS {
                Some(ref mut errors) => errors,
                None => unreachable!(),
            }
        }
    }

    pub fn get() -> &'static Self {
        unsafe {
            if ERRORS.is_none() {
                ERRORS = Some(Errors::default());
            }
            match ERRORS {
                Some(ref errors) => errors,
                None => unreachable!(),
            }
        }
    }
}

pub fn set_vanilla_dir(dir: PathBuf) {
    let mut game = dir.clone();
    game.push("game");
    Errors::get_mut().vanilla_root = game;

    let mut clausewitz = dir.clone();
    clausewitz.push("clausewitz");
    Errors::get_mut().clausewitz_root = clausewitz;

    let mut jomini = dir;
    jomini.push("jomini");
    Errors::get_mut().jomini_root = jomini;
}

pub fn set_mod_root(root: PathBuf) {
    Errors::get_mut().mod_root = root;
}

pub fn add_loaded_mod_root(label: String, root: PathBuf) {
    Errors::get_mut().loaded_mods_labels.push(label);
    Errors::get_mut().loaded_mods.push(root);
}

pub fn log(mut report: LogReport) {
    let mut vec = Vec::new();
    report.pointers.drain(..).for_each(|pointer| {
        let index = vec.len();
        recursive_pointed_msg_expansion(&mut vec, &pointer);
        vec.insert(index, pointer);
    });
    report.pointers.extend(vec);
    Errors::get_mut().push_report(&report);
}

/// Expand `PointedMessage` recursively.
/// That is; for the given `PointedMessage`, follow its location's link until such link is no
/// longer available, adding a newly created `PointedMessage` to the given `Vec` for each linked
/// location.
fn recursive_pointed_msg_expansion(vec: &mut Vec<PointedMessage>, pointer: &PointedMessage) {
    if let Some(link) = &pointer.location.link {
        let from_here = PointedMessage {
            location: link.as_ref().into_loc(),
            length: 1,
            msg: Some("from here"),
        };
        let index = vec.len();
        recursive_pointed_msg_expansion(vec, &from_here);
        vec.insert(index, from_here);
    }
}

pub fn error<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    log(LogReport {
        lvl: LogLevel::new(Severity::Error, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![PointedMessage {
            location: eloc.into_loc(),
            length: 1,
            msg: None,
        }],
    });
}

pub fn error_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    let info = if info.is_empty() { None } else { Some(info) };
    log(LogReport {
        lvl: LogLevel::new(Severity::Error, Confidence::Reasonable),
        key,
        msg,
        info,
        pointers: vec![PointedMessage {
            location: eloc.into_loc(),
            length: 1,
            msg: None,
        }],
    });
}

pub fn warn<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    log(LogReport {
        lvl: LogLevel::new(Severity::Warning, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![PointedMessage {
            location: eloc.into_loc(),
            length: 1,
            msg: None,
        }],
    });
}

pub fn warn2<E: ErrorLoc, F: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, eloc2: F, msg2: &str) {
    log(LogReport {
        lvl: LogLevel::new(Severity::Warning, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![
            PointedMessage {
                location: eloc.into_loc(),
                length: 1,
                msg: None,
            },
            PointedMessage {
                location: eloc2.into_loc(),
                length: 1,
                msg: Some(msg2),
            },
        ],
    });
}

pub fn warn3<E: ErrorLoc, E2: ErrorLoc, E3: ErrorLoc>(
    eloc: E,
    key: ErrorKey,
    msg: &str,
    eloc2: E2,
    msg2: &str,
    eloc3: E3,
    msg3: &str,
) {
    log(LogReport {
        lvl: LogLevel::new(Severity::Warning, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![
            PointedMessage {
                location: eloc.into_loc(),
                length: 1,
                msg: None,
            },
            PointedMessage {
                location: eloc2.into_loc(),
                length: 1,
                msg: Some(msg2),
            },
            PointedMessage {
                location: eloc3.into_loc(),
                length: 1,
                msg: Some(msg3),
            },
        ],
    });
}

pub fn warn_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    let info = if info.is_empty() { None } else { Some(info) };
    log(LogReport {
        lvl: LogLevel::new(Severity::Warning, Confidence::Reasonable),
        key,
        msg,
        info,
        pointers: vec![PointedMessage {
            location: eloc.into_loc(),
            length: 1,
            msg: None,
        }],
    });
}

pub fn advice<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    log(LogReport {
        lvl: LogLevel::new(Severity::Info, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![PointedMessage {
            location: eloc.into_loc(),
            length: 1,
            msg: None,
        }],
    });
}

pub fn advice2<E: ErrorLoc, F: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, eloc2: F, msg2: &str) {
    log(LogReport {
        lvl: LogLevel::new(Severity::Info, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![
            PointedMessage {
                location: eloc.into_loc(),
                length: 1,
                msg: None,
            },
            PointedMessage {
                location: eloc2.into_loc(),
                length: 1,
                msg: Some(msg2),
            },
        ],
    });
}

pub fn advice_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    let info = if info.is_empty() { None } else { Some(info) };
    log(LogReport {
        lvl: LogLevel::new(Severity::Info, Confidence::Reasonable),
        key,
        msg,
        info,
        pointers: vec![PointedMessage {
            location: eloc.into_loc(),
            length: 1,
            msg: None,
        }],
    });
}

pub fn warn_header(key: ErrorKey, msg: &str) {
    Errors::get_mut().push_header(key, msg);
}

pub fn warn_abbreviated<E: ErrorLoc>(eloc: E, key: ErrorKey) {
    Errors::get_mut().push_abbreviated(eloc, key);
}

pub fn will_log<E: ErrorLoc>(eloc: E, key: ErrorKey) -> bool {
    Errors::get()
        .filter
        .should_print(LogLevel::min(), &eloc.into_loc(), key)
}

pub trait ErrorLogger: Write {
    fn get_logs(&self) -> Option<String>;
}

impl ErrorLogger for Stderr {
    fn get_logs(&self) -> Option<String> {
        None
    }
}

impl ErrorLogger for Stdout {
    fn get_logs(&self) -> Option<String> {
        None
    }
}

impl ErrorLogger for Vec<u8> {
    fn get_logs(&self) -> Option<String> {
        Some(String::from_utf8_lossy(self).to_string())
    }
}

// =================================================================================================
// =============== Configuration (Output style):
// =================================================================================================

/// Override the default `OutputStyle`. (Controls ansi colors)
pub fn set_output_style(style: OutputStyle) {
    Errors::get_mut().styles = style;
}

/// Disable color in the output.
pub fn disable_ansi_colors() {
    Errors::get_mut().styles = OutputStyle::no_color();
}

pub fn set_max_line_length(max_line_length: usize) {
    Errors::get_mut().max_line_length = if max_line_length == 0 {
        None
    } else {
        Some(max_line_length)
    };
}

// =================================================================================================
// =============== Configuration (Filter):
// =================================================================================================

pub fn set_minimum_level(lvl: LogLevel) {
    Errors::get_mut().filter.min_level = lvl;
}

pub fn set_show_vanilla(v: bool) {
    Errors::get_mut().filter.show_vanilla = v;
}

pub fn set_show_loaded_mods(v: bool) {
    Errors::get_mut().filter.show_loaded_mods = v;
}

/// Add a key to the whitelist.
pub fn add_filter_rule_key(rules_type: RulesListType, key: ErrorKey) {
    Errors::get_mut().filter.get_or_create_rules_list(rules_type).add_rule_key(key);
}

/// Ignore this path for all keys
pub fn add_filter_rule_path(rules_type: RulesListType, path: PathBuf) {
    Errors::get_mut().filter.get_or_create_rules_list(rules_type)
        .add_rule_path(path);
}

pub fn add_filter_rule_key_in_path(rules_type: RulesListType, path: PathBuf, key: ErrorKey) {
    Errors::get_mut()
        .filter
        .get_or_create_rules_list(rules_type)
        .add_rule_key_in_path(path, key);
}
