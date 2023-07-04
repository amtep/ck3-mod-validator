use std::cell::RefCell;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::Result;
use fnv::FnvHashSet;
use strum::IntoEnumIterator;
use thiserror::Error;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::data::accessory::{Accessory, AccessoryVariation};
use crate::data::accolades::{AccoladeIcon, AccoladeName, AccoladeType};
use crate::data::activities::{
    ActivityIntent, ActivityLocale, ActivityType, GuestInviteRule, PulseAction,
};
use crate::data::amenities::Amenity;
use crate::data::artifacts::{
    ArtifactFeature, ArtifactFeatureGroup, ArtifactSlot, ArtifactTemplate, ArtifactType,
    ArtifactVisual,
};
use crate::data::assets::Assets;
use crate::data::bookmarks::{Bookmark, BookmarkGroup, BookmarkPortrait};
use crate::data::buildings::Building;
use crate::data::casusbelli::{CasusBelli, CasusBelliGroup};
use crate::data::character_templates::CharacterTemplate;
use crate::data::characters::Characters;
use crate::data::coa::{CoaDynamicDefinition, CoaTemplateList, Coas};
use crate::data::coadesigner::{
    CoaDesignerColorPalette, CoaDesignerColoredEmblem, CoaDesignerEmblemLayout, CoaDesignerPattern,
};
use crate::data::colors::NamedColor;
use crate::data::combat::CombatPhaseEvent;
use crate::data::combat_effects::CombatEffect;
use crate::data::council::{CouncilPosition, CouncilTask};
use crate::data::court_scene::{
    CourtSceneCulture, CourtSceneGroup, CourtSceneRole, CourtSceneSetting,
};
use crate::data::court_type::CourtType;
use crate::data::courtpos::{CourtPosition, CourtPositionCategory};
use crate::data::culture_history::CultureHistories;
use crate::data::cultures::{
    Culture, CultureAesthetic, CultureCreationName, CultureEra, CulturePillar, CultureTradition,
};
use crate::data::customloca::CustomLocalization;
use crate::data::data_binding::DataBindings;
use crate::data::deathreasons::DeathReason;
use crate::data::decisions::Decisions;
use crate::data::defines::Defines;
use crate::data::diarchies::{DiarchyMandate, DiarchyType};
use crate::data::difficulty::PlayableDifficultyInfo;
use crate::data::dna::Dna;
use crate::data::doctrines::Doctrines;
use crate::data::dynasties::Dynasties;
use crate::data::dynasty_legacies::{DynastyLegacy, DynastyPerk};
use crate::data::effect_localization::EffectLocalization;
use crate::data::election::Election;
use crate::data::environment::Environment;
use crate::data::ethnicity::Ethnicity;
use crate::data::event_themes::{EventBackground, EventTheme, EventTransition};
use crate::data::events::Events;
use crate::data::factions::Faction;
use crate::data::flavorization::Flavorization;
use crate::data::focus::Focus;
use crate::data::gameconcepts::GameConcepts;
use crate::data::gamerules::GameRule;
use crate::data::genes::Gene;
use crate::data::government::Government;
use crate::data::gui::Gui;
use crate::data::holdings::Holding;
use crate::data::holysites::HolySite;
use crate::data::hooks::Hook;
use crate::data::houses::Houses;
use crate::data::important_actions::ImportantAction;
use crate::data::innovations::Innovation;
use crate::data::inspirations::Inspiration;
use crate::data::interaction_cats::InteractionCategories;
use crate::data::interactions::Interactions;
use crate::data::laws::LawGroup;
use crate::data::lifestyles::Lifestyle;
use crate::data::localization::Localization;
use crate::data::maa::MenAtArmsTypes;
use crate::data::map_environment::MapEnvironment;
use crate::data::mapmodes::MapMode;
use crate::data::memories::MemoryType;
use crate::data::messages::Message;
use crate::data::modif::ModifierFormat;
use crate::data::modifiers::Modifier;
use crate::data::mottos::{Motto, MottoInsert};
use crate::data::music::Musics;
use crate::data::namelists::Namelists;
use crate::data::nickname::Nickname;
use crate::data::on_actions::OnActions;
use crate::data::opinions::OpinionModifier;
use crate::data::perks::Perk;
use crate::data::points_of_interest::PointOfInterest;
use crate::data::pool::{CharacterBackground, PoolSelector};
use crate::data::portrait::{
    PortraitAnimation, PortraitCamera, PortraitModifierGroup, PortraitModifierPack,
};
use crate::data::prov_history::ProvinceHistories;
use crate::data::provinces::Provinces;
use crate::data::regions::Region;
use crate::data::relations::Relation;
use crate::data::religions::{Religion, ReligionFamily};
use crate::data::schemes::Scheme;
use crate::data::scripted_animations::ScriptedAnimation;
use crate::data::scripted_costs::ScriptedCost;
use crate::data::scripted_effects::{Effect, Effects};
use crate::data::scripted_guis::ScriptedGui;
use crate::data::scripted_illustrations::ScriptedIllustration;
use crate::data::scripted_lists::ScriptedLists;
use crate::data::scripted_modifiers::ScriptedModifiers;
use crate::data::scripted_rules::ScriptedRule;
use crate::data::scripted_triggers::{Trigger, Triggers};
use crate::data::scriptvalues::ScriptValues;
use crate::data::secrets::Secret;
use crate::data::sound::Sounds;
use crate::data::stories::Story;
use crate::data::struggle::{Catalyst, Struggle};
use crate::data::suggestions::Suggestion;
use crate::data::terrain::Terrain;
use crate::data::title_history::TitleHistories;
use crate::data::titles::Titles;
use crate::data::traits::Traits;
use crate::data::travel::TravelOption;
use crate::data::trigger_localization::TriggerLocalization;
use crate::data::vassalcontract::VassalContract;
use crate::data::vassalstance::VassalStance;
use crate::db::{Db, DbKind};
use crate::dds::DdsFiles;
use crate::fileset::{FileEntry, FileKind, Fileset};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::report::{error, ignore_key, ignore_key_for, ignore_path, set_output_style, warn};
use crate::report::{ErrorKey, OutputStyle, Severity};
use crate::rivers::Rivers;
use crate::token::{Loc, Token};

#[derive(Debug, Error)]
pub enum FilesError {
    #[error("Could not read CK3 game files at {path}")]
    VanillaUnreadable {
        path: PathBuf,
        source: walkdir::Error,
    },
    #[error("Could not read mod files at {path}")]
    ModUnreadable {
        path: PathBuf,
        source: walkdir::Error,
    },
    #[error("Could not read config file at {path}")]
    ConfigUnreadable { path: PathBuf },
}

#[derive(Debug)]
pub struct Everything {
    /// Config from file
    config: Block,

    warned_defines: RefCell<FnvHashSet<String>>,

    /// The CK3 and mod files
    pub fileset: Fileset,

    pub dds: DdsFiles,

    pub database: Db,

    /// Processed localization files
    pub localization: Localization,

    pub scripted_lists: ScriptedLists,

    pub defines: Defines,

    /// Processed event files
    pub events: Events,

    /// Processed decision files
    pub decisions: Decisions,

    pub scripted_modifiers: ScriptedModifiers,
    pub on_actions: OnActions,

    /// Processed character interaction files
    pub interactions: Interactions,
    pub interaction_cats: InteractionCategories,

    /// Processed map data
    pub provinces: Provinces,

    /// Processed history/provinces data
    pub province_histories: ProvinceHistories,

    /// Processed game concepts
    pub gameconcepts: GameConcepts,

    /// Landed titles
    pub titles: Titles,

    pub dynasties: Dynasties,
    pub houses: Houses,
    pub characters: Characters,

    /// Cultural name lists
    pub namelists: Namelists,

    pub scriptvalues: ScriptValues,

    pub triggers: Triggers,
    pub effects: Effects,

    pub traits: Traits,

    pub title_history: TitleHistories,

    pub doctrines: Doctrines,

    pub menatarmstypes: MenAtArmsTypes,

    pub gui: Gui,
    pub data_bindings: DataBindings,

    pub assets: Assets,
    pub sounds: Sounds,
    pub music: Musics,

    pub coas: Coas,
    pub culture_history: CultureHistories,
}

impl Everything {
    pub fn new(
        vanilla_dir: &Path,
        mod_root: &Path,
        replace_paths: Vec<PathBuf>,
    ) -> Result<Self, FilesError> {
        let mut fileset = Fileset::new(
            vanilla_dir.to_path_buf(),
            mod_root.to_path_buf(),
            replace_paths,
        );

        let config_file = mod_root.join("ck3-tiger.conf");
        let config = if config_file.is_file() {
            Self::_read_config(&config_file)
                .ok_or(FilesError::ConfigUnreadable { path: config_file })?
        } else {
            Block::new(Loc::for_file(Rc::new(config_file), FileKind::Mod))
        };

        fileset.config(config.clone());

        fileset.scan_all()?;
        fileset.finalize();

        Ok(Everything {
            fileset,
            dds: DdsFiles::default(),
            config,
            warned_defines: RefCell::new(FnvHashSet::default()),
            database: Db::default(),
            localization: Localization::default(),
            scripted_lists: ScriptedLists::default(),
            defines: Defines::default(),
            events: Events::default(),
            decisions: Decisions::default(),
            scripted_modifiers: ScriptedModifiers::default(),
            on_actions: OnActions::default(),
            interactions: Interactions::default(),
            interaction_cats: InteractionCategories::default(),
            provinces: Provinces::default(),
            province_histories: ProvinceHistories::default(),
            gameconcepts: GameConcepts::default(),
            titles: Titles::default(),
            dynasties: Dynasties::default(),
            houses: Houses::default(),
            characters: Characters::default(),
            namelists: Namelists::default(),
            scriptvalues: ScriptValues::default(),
            triggers: Triggers::default(),
            effects: Effects::default(),
            traits: Traits::default(),
            title_history: TitleHistories::default(),
            doctrines: Doctrines::default(),
            menatarmstypes: MenAtArmsTypes::default(),
            gui: Gui::default(),
            data_bindings: DataBindings::default(),
            assets: Assets::default(),
            sounds: Sounds::default(),
            music: Musics::default(),
            coas: Coas::default(),
            culture_history: CultureHistories::default(),
        })
    }

    fn _read_config(path: &Path) -> Option<Block> {
        let entry = FileEntry::new(path.to_path_buf(), FileKind::Mod);
        PdxFile::read_no_bom(&entry, path)
    }

    pub fn fullpath(&self, entry: &FileEntry) -> PathBuf {
        self.fileset.fullpath(entry)
    }

    pub fn load_errorkey_config(&self) {
        for block in self.config.get_field_blocks("ignore") {
            let keynames = block.get_field_values("key");

            let mut keys = Vec::new();
            for keyname in keynames {
                let key = match keyname.as_str().parse() {
                    Ok(key) => key,
                    Err(e) => {
                        warn(keyname, ErrorKey::Config, &format!("{e:#}"));
                        continue;
                    }
                };
                keys.push(key);
            }

            let pathnames = block.get_field_values("file");
            if pathnames.is_empty() {
                for key in keys {
                    ignore_key(key);
                }
            } else if keys.is_empty() {
                for path in pathnames {
                    ignore_path(PathBuf::from(path.as_str()));
                }
            } else {
                for pathname in pathnames {
                    for &key in &keys {
                        ignore_key_for(PathBuf::from(pathname.as_str()), key);
                    }
                }
            }
        }
    }

    /// Load the `OutputStyle` settings from the config.
    /// Returns None if no settings are defined.
    /// Otherwise, returns the overwritten `OutputStyles`.
    ///
    /// Note that the settings from the config can still be overridden
    /// by supplying the --no-color flag.
    fn load_output_styles(&self) -> Option<OutputStyle> {
        let block = self.config.get_field_block("output_style")?;
        if !block.get_field_bool("enable").unwrap_or(true) {
            return Some(OutputStyle::no_color());
        }
        let mut style = OutputStyle::default();
        for severity in Severity::iter() {
            if let Some(error_block) =
                block.get_field_block(format!("{severity}").to_ascii_lowercase().as_str())
            {
                if let Some(color) = error_block.get_field_value("color") {
                    style.set(severity, color.as_str());
                }
            }
        }
        Some(style)
    }

    /// A helper function for categories of items that follow the usual pattern of
    /// `.txt` files containing a block with definitions
    pub fn load_pdx_items<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block),
    {
        self.load_pdx_items_ext(itype, add, ".txt");
    }

    /// Like `load_pdx_items` but does not complain about a missing BOM
    pub fn load_pdx_items_optional_bom<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block),
    {
        let subpath = PathBuf::from(itype.path());
        for entry in self.fileset.get_files_under(&subpath) {
            if entry.filename().to_string_lossy().ends_with(".txt") {
                if let Some(mut block) =
                    PdxFile::read_optional_bom(entry, &self.fileset.fullpath(entry))
                {
                    for (key, block) in block.drain_definitions_warn() {
                        add(&mut self.database, key, block);
                    }
                }
            }
        }
    }

    /// A helper function for categories of items that follow the usual pattern of
    /// `.txt` files containing a block with definitions
    pub fn load_pdx_items_ext<F>(&mut self, itype: Item, add: F, ext: &str)
    where
        F: Fn(&mut Db, Token, Block),
    {
        let subpath = PathBuf::from(itype.path());
        for entry in self.fileset.get_files_under(&subpath) {
            if entry.filename().to_string_lossy().ends_with(ext) {
                if let Some(mut block) = PdxFile::read(entry, &self.fileset.fullpath(entry)) {
                    for (key, block) in block.drain_definitions_warn() {
                        add(&mut self.database, key, block);
                    }
                }
            }
        }
    }

    /// A helper function for categories of items that are unusual in having each item in one file.
    pub fn load_pdx_files_optional_bom<F>(&mut self, itype: Item, add: F)
    where
        F: Fn(&mut Db, Token, Block),
    {
        let subpath = PathBuf::from(itype.path());
        for entry in self.fileset.get_files_under(&subpath) {
            let filename = entry.filename().to_string_lossy();
            if let Some(key) = filename.strip_suffix(".txt") {
                if let Some(block) =
                    PdxFile::read_optional_bom(entry, &self.fileset.fullpath(entry))
                {
                    let key = Token::new(key.to_string(), Loc::for_entry(entry));
                    add(&mut self.database, key, block);
                }
            }
        }
    }

    pub fn load_output_settings(&self) {
        self.load_errorkey_config();
        if let Some(style) = self.load_output_styles() {
            set_output_style(style);
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn load_all(&mut self) {
        self.fileset.handle(&mut self.dds);
        self.fileset.handle(&mut self.localization);
        self.fileset.handle(&mut self.scripted_lists);
        self.fileset.handle(&mut self.defines);
        self.fileset.handle(&mut self.events);
        self.fileset.handle(&mut self.decisions);
        self.fileset.handle(&mut self.scripted_modifiers);
        self.fileset.handle(&mut self.on_actions);
        self.fileset.handle(&mut self.interactions);
        self.fileset.handle(&mut self.interaction_cats);
        self.fileset.handle(&mut self.provinces);
        self.fileset.handle(&mut self.province_histories);
        self.fileset.handle(&mut self.gameconcepts);
        self.load_pdx_items(Item::Religion, Religion::add);
        self.load_pdx_items(Item::ReligionFamily, ReligionFamily::add);
        self.fileset.handle(&mut self.titles);
        self.fileset.handle(&mut self.dynasties);
        self.fileset.handle(&mut self.houses);
        self.fileset.handle(&mut self.characters);
        self.fileset.handle(&mut self.namelists);
        self.fileset.handle(&mut self.scriptvalues);
        self.fileset.handle(&mut self.triggers);
        self.fileset.handle(&mut self.effects);
        self.fileset.handle(&mut self.traits);
        self.load_pdx_items(Item::Lifestyle, Lifestyle::add);
        self.load_pdx_items(Item::CourtPositionCategory, CourtPositionCategory::add);
        self.load_pdx_items(Item::CourtPosition, CourtPosition::add);
        self.fileset.handle(&mut self.title_history);
        self.fileset.handle(&mut self.doctrines);
        self.fileset.handle(&mut self.menatarmstypes);
        self.load_pdx_items(Item::EventTheme, EventTheme::add);
        self.load_pdx_items(Item::EventBackground, EventBackground::add);
        self.load_pdx_items(Item::EventTransition, EventTransition::add);
        self.fileset.handle(&mut self.gui);
        self.fileset.handle(&mut self.data_bindings);
        self.fileset.handle(&mut self.assets);
        self.fileset.handle(&mut self.sounds);
        self.fileset.handle(&mut self.music);
        self.load_pdx_items(Item::ScriptedRule, ScriptedRule::add);
        self.load_pdx_items(Item::Faction, Faction::add);
        self.load_pdx_items(Item::Relation, Relation::add);
        self.load_pdx_items(Item::Terrain, Terrain::add);
        self.load_pdx_items(Item::Region, Region::add);
        self.load_pdx_items(Item::ScriptedGui, ScriptedGui::add);
        self.load_pdx_items(Item::GeneCategory, Gene::add);
        self.load_pdx_items(Item::Amenity, Amenity::add);
        self.load_pdx_items(Item::CasusBelliGroup, CasusBelliGroup::add);
        self.load_pdx_items(Item::CasusBelli, CasusBelli::add);
        self.load_pdx_items(Item::Holding, Holding::add);
        self.load_pdx_items(Item::TriggerLocalization, TriggerLocalization::add);
        self.load_pdx_items(Item::EffectLocalization, EffectLocalization::add);
        self.load_pdx_items(Item::Focus, Focus::add);
        self.load_pdx_items(Item::Perk, Perk::add);
        self.load_pdx_items(Item::OpinionModifier, OpinionModifier::add);
        self.load_pdx_items(Item::CharacterTemplate, CharacterTemplate::add);
        self.load_pdx_items(Item::DeathReason, DeathReason::add);
        self.load_pdx_items(Item::ArtifactSlot, ArtifactSlot::add);
        self.load_pdx_items(Item::ArtifactType, ArtifactType::add);
        self.load_pdx_items(Item::ArtifactTemplate, ArtifactTemplate::add);
        self.load_pdx_items(Item::ArtifactVisual, ArtifactVisual::add);
        self.load_pdx_items(Item::ArtifactFeature, ArtifactFeature::add);
        self.load_pdx_items(Item::ArtifactFeatureGroup, ArtifactFeatureGroup::add);
        self.load_pdx_items(Item::Nickname, Nickname::add);
        self.load_pdx_items(Item::CustomLocalization, CustomLocalization::add);
        self.load_pdx_items(Item::Building, Building::add);
        Building::finalize(&mut self.database);
        self.load_pdx_items(Item::Culture, Culture::add);
        self.load_pdx_items(Item::CultureEra, CultureEra::add);
        self.load_pdx_items(Item::CulturePillar, CulturePillar::add);
        self.load_pdx_items(Item::CultureTradition, CultureTradition::add);
        self.load_pdx_items(Item::CultureAesthetic, CultureAesthetic::add);
        self.load_pdx_items(Item::CultureCreationName, CultureCreationName::add);
        self.load_pdx_items(Item::NamedColor, NamedColor::add);
        self.load_pdx_items(Item::Innovation, Innovation::add);
        self.load_pdx_items(Item::Accessory, Accessory::add);
        self.load_pdx_items(Item::AccessoryVariation, AccessoryVariation::add);
        self.load_pdx_items(Item::PortraitModifierGroup, PortraitModifierGroup::add);
        self.load_pdx_items_ext(
            Item::PortraitModifierPack,
            PortraitModifierPack::add,
            ".modifierpack",
        );
        self.load_pdx_items(Item::PortraitCamera, PortraitCamera::add);
        self.load_pdx_items(Item::AccoladeIcon, AccoladeIcon::add);
        self.load_pdx_items(Item::AccoladeName, AccoladeName::add);
        self.load_pdx_items(Item::AccoladeType, AccoladeType::add);
        self.load_pdx_items(Item::VassalStance, VassalStance::add);
        self.load_pdx_items(Item::Dna, Dna::add);
        self.load_pdx_items(Item::Bookmark, Bookmark::add);
        self.load_pdx_items(Item::BookmarkGroup, BookmarkGroup::add);
        self.load_pdx_items_optional_bom(Item::BookmarkPortrait, BookmarkPortrait::add);
        self.load_pdx_items(Item::Ethnicity, Ethnicity::add);
        self.load_pdx_items(Item::Modifier, Modifier::add);
        self.load_pdx_items(Item::GovernmentType, Government::add);
        self.load_pdx_items(Item::Hook, Hook::add);
        self.load_pdx_items(Item::CouncilPosition, CouncilPosition::add);
        self.load_pdx_items(Item::CouncilTask, CouncilTask::add);
        self.load_pdx_items(Item::PoolSelector, PoolSelector::add);
        self.load_pdx_items(Item::CharacterBackground, CharacterBackground::add);
        self.load_pdx_items(Item::HolySite, HolySite::add);
        self.fileset.handle(&mut self.coas);
        self.load_pdx_items_optional_bom(Item::CoaTemplateList, CoaTemplateList::add);
        self.load_pdx_items(Item::CoaDynamicDefinition, CoaDynamicDefinition::add);
        self.load_pdx_items(Item::Environment, Environment::add);
        self.load_pdx_items(Item::Struggle, Struggle::add);
        self.load_pdx_items(Item::Catalyst, Catalyst::add);
        self.load_pdx_items(Item::ImportantAction, ImportantAction::add);
        self.load_pdx_items(Item::Suggestion, Suggestion::add);
        self.load_pdx_items(Item::Scheme, Scheme::add);
        self.load_pdx_items(Item::ModifierFormat, ModifierFormat::add);
        self.load_pdx_items(Item::MemoryType, MemoryType::add);
        self.load_pdx_items(Item::MapMode, MapMode::add);
        self.load_pdx_items(Item::VassalContract, VassalContract::add);
        self.load_pdx_items(Item::CourtType, CourtType::add);
        self.load_pdx_items(Item::Secret, Secret::add);
        self.load_pdx_items(Item::ActivityType, ActivityType::add);
        self.load_pdx_items(Item::ActivityLocale, ActivityLocale::add);
        self.load_pdx_items(Item::ActivityIntent, ActivityIntent::add);
        self.load_pdx_items(Item::GuestInviteRule, GuestInviteRule::add);
        self.load_pdx_items(Item::PulseAction, PulseAction::add);
        self.load_pdx_items(Item::ScriptedAnimation, ScriptedAnimation::add);
        self.load_pdx_items(Item::CourtSceneCulture, CourtSceneCulture::add);
        self.load_pdx_items(Item::CourtSceneGroup, CourtSceneGroup::add);
        self.load_pdx_items(Item::CourtSceneRole, CourtSceneRole::add);
        self.load_pdx_files_optional_bom(Item::CourtSceneSetting, CourtSceneSetting::add);
        self.load_pdx_files_optional_bom(Item::MapEnvironment, MapEnvironment::add);
        self.load_pdx_items(Item::PortraitAnimation, PortraitAnimation::add);
        self.load_pdx_items(Item::GameRule, GameRule::add);
        self.load_pdx_items(Item::TravelOption, TravelOption::add);
        self.load_pdx_items(Item::Story, Story::add);
        self.load_pdx_items(Item::LawGroup, LawGroup::add);
        self.load_pdx_items(Item::SuccessionElection, Election::add);
        self.load_pdx_items(Item::DiarchyType, DiarchyType::add);
        self.load_pdx_items(Item::DiarchyMandate, DiarchyMandate::add);
        self.load_pdx_items(Item::Inspiration, Inspiration::add);
        self.load_pdx_items(
            Item::CoaDesignerColoredEmblem,
            CoaDesignerColoredEmblem::add,
        );
        self.load_pdx_items(Item::CoaDesignerColorPalette, CoaDesignerColorPalette::add);
        self.load_pdx_items(Item::CoaDesignerEmblemLayout, CoaDesignerEmblemLayout::add);
        self.load_pdx_items(Item::CoaDesignerPattern, CoaDesignerPattern::add);
        self.load_pdx_items(Item::PointOfInterest, PointOfInterest::add);
        self.load_pdx_items(Item::DynastyLegacy, DynastyLegacy::add);
        self.load_pdx_items(Item::DynastyPerk, DynastyPerk::add);
        self.load_pdx_items(Item::CombatEffect, CombatEffect::add);
        self.load_pdx_items(Item::ScriptedIllustration, ScriptedIllustration::add);
        self.load_pdx_items(Item::Flavorization, Flavorization::add);
        self.fileset.handle(&mut self.culture_history);
        self.load_pdx_items(Item::Motto, Motto::add);
        self.load_pdx_items(Item::MottoInsert, MottoInsert::add);
        self.load_pdx_items(Item::CombatPhaseEvent, CombatPhaseEvent::add);
        self.load_pdx_items(Item::ScriptedCost, ScriptedCost::add);
        self.load_pdx_items(Item::PlayableDifficultyInfo, PlayableDifficultyInfo::add);
        self.load_pdx_items(Item::Message, Message::add);
    }

    pub fn validate_all(&mut self) {
        self.fileset.validate(self);
        self.localization.validate(self);
        self.scripted_lists.validate(self);
        self.defines.validate(self);
        // scripted items go early because they update their scope context info
        self.scriptvalues.validate(self);
        self.triggers.validate(self);
        self.effects.validate(self);
        self.scripted_modifiers.validate(self);
        self.on_actions.validate(self);
        self.events.validate(self);
        self.decisions.validate(self);
        self.interactions.validate(self);
        self.interaction_cats.validate(self);
        self.provinces.validate(self);
        self.province_histories.validate(self);
        self.gameconcepts.validate(self);
        self.titles.validate(self);
        self.dynasties.validate(self);
        self.houses.validate(self);
        self.characters.validate(self);
        self.namelists.validate(self);
        self.traits.validate(self);
        self.title_history.validate(self);
        self.doctrines.validate(self);
        self.menatarmstypes.validate(self);
        self.gui.validate(self);
        self.data_bindings.validate(self);
        self.assets.validate(self);
        self.sounds.validate(self);
        self.music.validate(self);
        self.coas.validate(self);
        self.culture_history.validate(self);
        self.database.validate(self);
    }

    pub fn check_rivers(&mut self) {
        let mut rivers = Rivers::default();
        self.fileset.handle(&mut rivers);
        rivers.validate(self);
    }

    pub fn check_pod(&mut self) {
        self.province_histories.check_pod_faiths(self, &self.titles);
        self.characters.check_pod_flags(self);
        self.localization.check_pod_loca(self);
    }

    pub fn check_unused(&mut self) {
        self.localization.check_unused(self);
        self.fileset.check_unused_dds(self);
    }

    pub fn item_has_property(&self, itype: Item, key: &str, property: &str) -> bool {
        self.database.has_property(itype, key, property, self)
    }

    pub fn item_exists(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::ActivityState => ACTIVITY_STATES.contains(&key),
            Item::ArtifactHistory => ARTIFACT_HISTORY.contains(&key),
            Item::ArtifactRarity => ARTIFACT_RARITY.contains(&key),
            Item::Asset => self.assets.asset_exists(key),
            Item::BlendShape => self.assets.blend_shape_exists(key),
            Item::Character => self.characters.exists(key),
            Item::Coa => self.coas.exists(key),
            Item::CoaTemplate => self.coas.template_exists(key),
            Item::DangerType => DANGER_TYPES.contains(&key),
            Item::Decision => self.decisions.exists(key),
            Item::Define => self.defines.exists(key),
            Item::Dlc => DLC.contains(&key),
            Item::DlcFeature => DLC_FEATURES.contains(&key),
            Item::Doctrine => self.doctrines.exists(key),
            Item::DoctrineParameter => self.doctrines.parameter_exists(key),
            Item::Dynasty => self.dynasties.exists(key),
            Item::Entity => self.assets.entity_exists(key),
            Item::Event => self.events.exists(key),
            Item::EventNamespace => self.events.namespace_exists(key),
            Item::File => self.fileset.exists(key),
            Item::GameConcept => self.gameconcepts.exists(key),
            Item::GeneAttribute => self.assets.attribute_exists(key),
            Item::GeneticConstraint => self.traits.constraint_exists(key),
            Item::House => self.houses.exists(key),
            Item::Interaction => self.interactions.exists(key),
            Item::InteractionCategory => self.interaction_cats.exists(key),
            Item::Localization => self.localization.exists(key),
            Item::MenAtArms => self.menatarmstypes.exists(key),
            Item::MenAtArmsBase => self.menatarmstypes.base_exists(key),
            Item::Music => self.music.exists(key),
            Item::NameList => self.namelists.exists(key),
            Item::OnAction => self.on_actions.exists(key),
            Item::Pdxmesh => self.assets.mesh_exists(key),
            Item::PrisonType => PRISON_TYPES.contains(&key),
            Item::Province => self.provinces.exists(key),
            Item::RewardItem => REWARD_ITEMS.contains(&key),
            Item::ScriptedEffect => self.effects.exists(key),
            Item::ScriptedList => self.scripted_lists.exists(key),
            Item::ScriptedModifier => self.scripted_modifiers.exists(key),
            Item::ScriptedTrigger => self.triggers.exists(key),
            Item::ScriptValue => self.scriptvalues.exists(key),
            Item::Sexuality => SEXUALITIES.contains(&key),
            Item::Skill => SKILLS.contains(&key),
            Item::Sound => self.sounds.exists(key),
            Item::TextureFile => self.assets.texture_exists(key),
            Item::Title => self.titles.exists(key),
            Item::TitleHistory => self.title_history.exists(key),
            Item::TitleHistoryType => TITLE_HISTORY_TYPES.contains(&key),
            Item::Trait => self.traits.exists(key),
            Item::TraitFlag => self.traits.flag_exists(key),
            Item::TraitTrack => self.traits.track_exists(key),
            Item::TraitCategory => TRAIT_CATEGORIES.contains(&key),
            _ => self.database.exists(itype, key),
        }
    }

    pub fn item_used(&self, itype: Item, key: &str) {
        match itype {
            Item::File => self.fileset.mark_used(key),
            Item::Localization => self.localization.mark_used(key),
            _ => (),
        }
    }

    pub fn verify_exists(&self, itype: Item, token: &Token) {
        self.verify_exists_implied(itype, token.as_str(), token);
    }

    pub fn verify_exists_implied(&self, itype: Item, key: &str, token: &Token) {
        match itype {
            Item::File => self.fileset.verify_exists_implied(key, token),
            Item::Localization => self.localization.verify_exists_implied(key, token),
            Item::Music => self.music.verify_exists_implied(key, token),
            Item::Province => self.provinces.verify_exists_implied(key, token),
            Item::Sound => self.sounds.verify_exists_implied(key, token, self),
            Item::TextureFile => {
                if let Some(entry) = self.assets.get_texture(key) {
                    // TODO: avoid allocating a string here
                    self.fileset.mark_used(&entry.path().to_string_lossy());
                } else {
                    let msg = format!("no texture file {key} anywhere under {}", itype.path());
                    error(token, ErrorKey::MissingFile, &msg);
                }
            }
            _ => {
                if !self.item_exists(itype, key) {
                    let msg = format!("{} {} not defined in {}", itype, key, itype.path());
                    error(token, ErrorKey::MissingItem, &msg);
                }
            }
        }
    }

    pub fn validate_use(&self, itype: Item, key: &Token, block: &Block) {
        self.database.validate_use(itype, key, block, self);
    }

    pub fn validate_call(&self, itype: Item, key: &Token, block: &Block, sc: &mut ScopeContext) {
        self.database.validate_call(itype, key, block, self, sc);
    }

    pub fn get_item<T: DbKind>(&self, itype: Item, key: &str) -> Option<(&Token, &Block, &T)> {
        self.database.get_item(itype, key)
    }

    pub fn get_key_block(&self, itype: Item, key: &str) -> Option<(&Token, &Block)> {
        self.database.get_key_block(itype, key)
    }

    pub fn get_trigger(&self, key: &Token) -> Option<&Trigger> {
        if let Some(trigger) = self.triggers.get(key.as_str()) {
            Some(trigger)
        } else if let Some(trigger) = self.events.get_trigger(key) {
            Some(trigger)
        } else {
            None
        }
    }

    pub fn get_effect(&self, key: &Token) -> Option<&Effect> {
        if let Some(effect) = self.effects.get(key.as_str()) {
            Some(effect)
        } else if let Some(effect) = self.events.get_effect(key) {
            Some(effect)
        } else {
            None
        }
    }

    pub fn get_defined_string(&self, key: &str) -> Option<&Token> {
        self.defines.get_string(key)
    }

    pub fn get_defined_string_warn(&self, token: &Token, key: &str) -> Option<&Token> {
        let result = self.get_defined_string(key);
        let mut cache = self.warned_defines.borrow_mut();
        if result.is_none() && !cache.contains(key) {
            warn(
                token,
                ErrorKey::MissingItem,
                &format!("{key} not defined in common/defines/"),
            );
            cache.insert(key.to_string());
        }
        result
    }
}

/// LAST UPDATED VERSION 1.9.2
const ACTIVITY_STATES: &[&str] = &["passive", "travel", "active"];

/// LAST UPDATED VERSION 1.9.2
const ARTIFACT_HISTORY: &[&str] = &[
    "created_before_history",
    "created",
    "prize_created",
    "discovered",
    "creator_discovered",
    "claimed_by_house",
    "given",
    "stolen",
    "inherited",
    "conquest",
    "taken_in_siege",
    "taken_in_battle",
    "won_in_duel",
    "purchased",
    "prize_awarded",
    "ransomed",
    "reforged",
];

/// LAST UPDATED VERSION 1.9.2
// TODO: parse it from dlc_metadata/ ? Unfortunately Tours and Tournaments
// is an exception.
const DLC: &[&str] = &[
    "Fashion of the Abbasid Court",
    "The Northern Lords",
    "Garments of the Holy Roman Empire",
    "The Fate of Iberia",
    "The Royal Court",
    "Friends and Foes",
    "tours_and_tournaments",
    "Elegance of the Empire",
];

/// LAST UPDATED VERSION 1.9.2
const DLC_FEATURES: &[&str] = &[
    "garments_of_the_hre",
    "fashion_of_the_abbasid_court",
    "the_northern_lords",
    "hybridize_culture",
    "diverge_culture",
    "royal_court",
    "reform_culture",
    "court_artifacts",
    "the_fate_of_iberia",
    "friends_and_foes",
    "tours_and_tournaments",
    "advanced_activities",
    "accolades",
    "elegance_of_the_empire",
];

/// LAST UPDATED VERSION 1.9.2
const REWARD_ITEMS: &[&str] = &["newsletter_crown"];

/// LAST UPDATED VERSION 1.9.2
const PRISON_TYPES: &[&str] = &["dungeon", "house_arrest"];

/// LAST UPDATED VERSION 1.9.2
const SKILLS: &[&str] = &[
    "diplomacy",
    "intrigue",
    "learning",
    "martial",
    "prowess",
    "stewardship",
];

/// LAST UPDATED VERSION 1.9.2
const SEXUALITIES: &[&str] = &["heterosexual", "homosexual", "bisexual", "asexual", "none"];

/// LAST UPDATED VERSION 1.9.2
const TITLE_HISTORY_TYPES: &[&str] = &[
    "conquest",
    "conquest_holy_war",
    "conquest_claim",
    "conquest_populist",
    "election",
    "inheritance",
    "abdication",
    "created",
    "destroyed",
    "usurped",
    "granted",
    "revoked",
    "independency",
    "leased_out",
    "lease_revoked",
    "returned",
    "faction_demand",
    "swear_fealty",
];

/// LAST UPDATED VERSION 1.9.2
const TRAIT_CATEGORIES: &[&str] = &[
    "personality",
    "education",
    "childhood",
    "commander",
    "winter_commander",
    "lifestyle",
    "court_type",
    "fame",
    "health",
];

/// LAST UPDATED VERSION 1.9.2
const DANGER_TYPES: &[&str] = &[
    "default",
    "battle",
    "raid",
    "siege",
    "army",
    "occupation",
    "county_control",
    "county_opinion",
    "owner_opinion",
];

/// LAST UPDATED VERSION 1.9.2
const ARTIFACT_RARITY: &[&str] = &["common", "masterwork", "famed", "illustrious"];
