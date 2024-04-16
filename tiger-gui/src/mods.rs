use std::fmt;
use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use enum_map::EnumMap;
use iced::alignment::Horizontal;
use iced::widget::{button, column, container, horizontal_rule, scrollable, text, Column};
use iced::{theme, Element, Length};
use regex::Regex;
use strum::IntoEnumIterator;

use crate::game::Game;
use crate::message::Message;

#[derive(Debug)]
pub(crate) struct Mods {
    /// Mods to choose from when running Tiger, in the order they should be displayed to the user.
    /// Invariant: Every game has an entry in the map.
    game_mods: EnumMap<Game, Vec<Mod>>,
}

impl Default for Mods {
    fn default() -> Self {
        Self { game_mods: Game::iter().map(|game| (game, Vec::new())).collect() }
    }
}

impl Mods {
    /// Scan the mods that are installed locally in each game's mod directory (so not counting the
    /// ones installed from the workshop).
    // TODO: load mods from persistent settings first, and then add newly scanned mods and skip
    // scanned mods that were removed by the user.
    pub(crate) fn load() -> Self {
        let mut mods = Self::default();

        for game in Game::iter() {
            mods.game_mods[game] = Mod::enumerate_local(game);
        }

        mods
    }

    pub(crate) fn view(&self) -> Column<Message> {
        let mut game_sections = column![].spacing(10);
        for game in Game::iter() {
            let mut game_section = column![text(game.fullname()), horizontal_rule(1)];
            for a_mod in &self.game_mods[game] {
                game_section = game_section.push(container(a_mod.view()).padding(2));
            }
            game_sections = game_sections.push(game_section);
        }

        column![text("Mods"), container(scrollable(game_sections))]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mod {
    pub game: Game,
    pub name: String,
    pub version: String,
    /// Equal to `dir` for vic3, or the path to the `.mod` file for ck3 or imperator
    pub locator: PathBuf,
}

impl Mod {
    /// List the locally installed mods for the given game.
    /// Ignore any errors encountered in the process; just return the mods that were readable.
    fn enumerate_local(game: Game) -> Vec<Self> {
        let mut mods = Vec::new();
        if let Some(mod_dir) = game.find_mod_dir() {
            if let Ok(entries) = read_dir(mod_dir) {
                for entry in entries.flatten() {
                    match game {
                        Game::Ck3 | Game::Imperator => {
                            if entry.file_name().to_string_lossy().ends_with(".mod")
                                && !entry.file_name().to_string_lossy().starts_with("ugc_")
                            {
                                if let Ok(the_mod) = Self::from_descriptor(game, &entry.path()) {
                                    mods.push(the_mod);
                                }
                            }
                        }
                        Game::Vic3 => {
                            if let Ok(the_mod) = Self::from_metadata(game, &entry.path()) {
                                mods.push(the_mod);
                            }
                        }
                    }
                }
            }
        }
        mods.sort();
        mods
    }

    /// Construct a `Mod` from reading a `.mod` file.
    fn from_descriptor(game: Game, descriptor_path: &Path) -> Result<Self> {
        let descriptor = read_to_string(descriptor_path)?;
        let name =
            if let Some(capture) = Regex::new("name\\s*=\\s*\"([^\"]+)\"")?.captures(&descriptor) {
                capture[1].to_owned()
            } else {
                bail!("no name field in mod descriptor");
            };
        let version = if let Some(capture) =
            Regex::new("version\\s*=\\s*\"([^\"]+)\"")?.captures(&descriptor)
        {
            capture[1].to_owned()
        } else {
            bail!("no version field in mod descriptor");
        };
        let path =
            if let Some(capture) = Regex::new("path\\s*=\\s*\"([^\"]+)\"")?.captures(&descriptor) {
                capture[1].to_owned()
            } else {
                bail!("no path field in mod descriptor");
            };
        let dir = if path.starts_with('/') {
            PathBuf::from(path)
        } else {
            // Relative paths are relative to the parent of the mod/ directory,
            // so do parent() twice: once to get rid of the .mod filename, then
            // again to get rid of the mod directory.
            descriptor_path.parent().unwrap().parent().unwrap().join(path)
        };
        if !dir.is_dir() {
            bail!("mod folder does not exist");
        }
        Ok(Mod { game, name, version, locator: descriptor_path.to_owned() })
    }

    /// Construct a `Mod` from reading a `metadata.json` file.
    fn from_metadata(game: Game, dir: &Path) -> Result<Self> {
        let metadata = read_to_string(dir.join(".metadata/metadata.json"))?;
        let value: serde_json::Value = serde_json::from_str(&metadata)?;
        if let (Some(name), Some(version)) = (value["name"].as_str(), value["version"].as_str()) {
            Ok(Mod {
                game,
                name: name.to_owned(),
                version: version.to_owned(),
                locator: dir.to_owned(),
            })
        } else {
            bail!("missing fields in .metadata/metadata.json");
        }
    }

    fn view(&self) -> Element<Message> {
        let contents =
            text(format!("{self}")).width(Length::Fill).horizontal_alignment(Horizontal::Center);
        button(contents)
            .on_press(Message::ShowResults(self.clone()))
            .style(theme::Button::Secondary)
            .into()
    }
}

impl fmt::Display for Mod {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} (v{})", self.name, self.version)
    }
}
