#![allow(non_camel_case_types)]

use std::str::FromStr;

use strum_macros::{Display, EnumString};

use Arg::*;
use Args::*;
use Datatype::*;

use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;

// Validate the "code" blocks in localization files and in the gui files.
// The include/ files are converted from the game's data_type_* output files.

include!("include/datatypes.rs");

#[derive(Copy, Clone, Debug)]
pub enum Arg {
    DType(Datatype),
    IType(Item),
}

#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, Debug)]
pub enum Args {
    NoArgs,
    Arg1(Arg),
    Arg2(Arg, Arg),
    Arg3(Arg, Arg, Arg),
    Arg4(Arg, Arg, Arg, Arg),
    Arg5(Arg, Arg, Arg, Arg, Arg),
}

impl Args {
    pub fn nargs(self) -> usize {
        match self {
            NoArgs => 0,
            Arg1(_) => 1,
            Arg2(_, _) => 2,
            Arg3(_, _, _) => 3,
            Arg4(_, _, _, _) => 4,
            Arg5(_, _, _, _, _) => 5,
        }
    }
}

pub enum LookupResult {
    NotFound,
    WrongType,
    Found(Args, Datatype),
}

pub fn lookup_global_promote(lookup_name: &str) -> Option<(Args, Datatype)> {
    if let Ok(idx) = GLOBAL_PROMOTES.binary_search_by_key(&lookup_name, |(name, _, _)| name) {
        let (_name, args, rtype) = GLOBAL_PROMOTES[idx];
        return Some((args, rtype));
    }

    // Datatypes can be used directly as global promotes, taking their value from the gui context.
    if let Ok(dtype) = Datatype::from_str(lookup_name) {
        return Some((Args::NoArgs, dtype));
    }

    None
}

pub fn lookup_global_function(lookup_name: &str) -> Option<(Args, Datatype)> {
    if let Ok(idx) = GLOBAL_FUNCTIONS.binary_search_by_key(&lookup_name, |(name, _, _)| name) {
        let (_name, args, rtype) = GLOBAL_FUNCTIONS[idx];
        return Some((args, rtype));
    }
    None
}

fn lookup_promote_or_function(
    lookup_name: &str,
    ltype: Datatype,
    global: &[(&str, Datatype, Args, Datatype)],
) -> LookupResult {
    let start = global.partition_point(|(name, _, _, _)| name < &lookup_name);
    let mut found_any = false;
    let mut possible_args = None;
    let mut possible_rtype = None;
    for (name, intype, args, rtype) in global.iter().skip(start) {
        if lookup_name != *name {
            break;
        }
        found_any = true;
        if ltype == Datatype::Unknown {
            if possible_rtype.is_none() {
                possible_args = Some(*args);
                possible_rtype = Some(*rtype);
            } else if possible_rtype != Some(*rtype) {
                possible_rtype = Some(Datatype::Unknown);
            }
        } else if ltype == *intype {
            return LookupResult::Found(*args, *rtype);
        }
    }

    if found_any {
        if ltype == Datatype::Unknown {
            LookupResult::Found(possible_args.unwrap(), possible_rtype.unwrap())
        } else {
            LookupResult::WrongType
        }
    } else {
        LookupResult::NotFound
    }
}

pub fn lookup_promote(lookup_name: &str, ltype: Datatype) -> LookupResult {
    lookup_promote_or_function(lookup_name, ltype, PROMOTES)
}

pub fn lookup_function(lookup_name: &str, ltype: Datatype) -> LookupResult {
    lookup_promote_or_function(lookup_name, ltype, FUNCTIONS)
}

/// Find an alternative datafunction to suggest when `lookup_name` has not been found.
/// This is a fairly expensive lookup.
/// Currently it only looks for different-case variants.
/// TODO: make it consider misspellings as well
pub fn lookup_alternative(
    lookup_name: &str,
    data: &Everything,
    first: std::primitive::bool,
    last: std::primitive::bool,
) -> Option<&'static str> {
    let lc = lookup_name.to_lowercase();
    if first {
        for (name, _, _) in GLOBAL_PROMOTES {
            if name.to_lowercase() == lc {
                return Some(name);
            }
        }
        if last {
            for (name, _, _) in GLOBAL_FUNCTIONS {
                if data.item_exists(Item::GameConcept, name) {
                    continue;
                }
                if name.to_lowercase() == lc {
                    return Some(name);
                }
            }
        }
    } else {
        for (name, _, _, _) in PROMOTES {
            if name.to_lowercase() == lc {
                return Some(name);
            }
        }
        if last {
            for (name, _, _, _) in FUNCTIONS {
                if name.to_lowercase() == lc {
                    return Some(name);
                }
            }
        }
    }
    None
}

pub fn scope_from_datatype(dtype: Datatype) -> Option<Scopes> {
    match dtype {
        Datatype::Character => Some(Scopes::Character),
        Datatype::Title => Some(Scopes::LandedTitle),
        Datatype::Activity => Some(Scopes::Activity),
        Datatype::Secret => Some(Scopes::Secret),
        Datatype::Province => Some(Scopes::Province),
        Datatype::Scheme => Some(Scopes::Scheme),
        Datatype::Combat => Some(Scopes::Combat),
        Datatype::CombatSide => Some(Scopes::CombatSide),
        Datatype::Faith => Some(Scopes::Faith),
        Datatype::GreatHolyWar => Some(Scopes::GreatHolyWar),
        Datatype::Religion => Some(Scopes::Religion),
        Datatype::War => Some(Scopes::War),
        Datatype::Story => Some(Scopes::StoryCycle),
        Datatype::CasusBelliItem => Some(Scopes::CasusBelli),
        Datatype::Dynasty => Some(Scopes::Dynasty),
        Datatype::DynastyHouse => Some(Scopes::DynastyHouse),
        Datatype::Faction => Some(Scopes::Faction),
        Datatype::Culture => Some(Scopes::Culture),
        Datatype::Army => Some(Scopes::Army),
        Datatype::HolyOrder => Some(Scopes::HolyOrder),
        Datatype::ActiveCouncilTask => Some(Scopes::CouncilTask),
        Datatype::MercenaryCompany => Some(Scopes::MercenaryCompany),
        Datatype::Artifact => Some(Scopes::Artifact),
        Datatype::Inspiration => Some(Scopes::Inspiration),
        Datatype::Struggle => Some(Scopes::Struggle),
        Datatype::CharacterMemory => Some(Scopes::CharacterMemory),
        Datatype::TravelPlan => Some(Scopes::TravelPlan),
        Datatype::Accolade => Some(Scopes::Accolade),
        Datatype::AccoladeType => Some(Scopes::AccoladeType),
        Datatype::Decision => Some(Scopes::Decision),
        Datatype::FaithDoctrine => Some(Scopes::Doctrine),
        Datatype::ActivityType => Some(Scopes::ActivityType),
        Datatype::CultureTradition => Some(Scopes::CultureTradition),
        Datatype::CulturePillar => Some(Scopes::CulturePillar),
        Datatype::GovernmentType => Some(Scopes::GovernmentType),
        Datatype::Trait => Some(Scopes::Trait),
        Datatype::VassalContract => Some(Scopes::VassalContract),
        Datatype::ObligationLevel => Some(Scopes::VassalObligationLevel),
        _ => None,
    }
}

const GLOBAL_PROMOTES: &[(&str, Args, Datatype)] = include!("include/data_global_promotes.rs");

const GLOBAL_FUNCTIONS: &[(&str, Args, Datatype)] = include!("include/data_global_functions.rs");

const PROMOTES: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");

const FUNCTIONS: &[(&str, Datatype, Args, Datatype)] = include!("include/data_functions.rs");
