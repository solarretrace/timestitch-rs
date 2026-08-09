////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Event record entries.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::Attribute;
use crate::TimeInterval;

// External library imports.
use serde::Deserialize;
use serde::Serialize;


// Standard library imports.
use std::collections::BTreeMap;
use std::path::PathBuf;


pub type EntryId = u64;

/// A indivisible record entry.
#[derive(Debug)]
pub struct Entry {
    /// The entries unique identifier.
    pub id: EntryId,
    /// The time interval over which the entry is valid.
    pub time: TimeInterval,
    /// The file source of the entry.
    
    pub source_path: PathBuf,
    /// The reference source of the entry.
    pub source_ref: Box<str>,
    /// The entry contents.
    pub attributes: BTreeMap<Box<str>, Box<dyn Attribute + 'static>>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum MatchSource {
    Default,
    Path {
        group: usize
    },
    Content {
        line: usize,
        group: usize
    },
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum MatchSourceAttribute {
    Default(MatchFormat),
    Path {
        group: usize,
        format: MatchFormat,
    },
    Content {
        line: usize,
        group: usize,
        format: MatchFormat,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum MatchFormat {
    U64,
    Time,
    Text,
}


impl MatchFormat {
    pub fn default_dyn(self) -> Box<dyn Attribute> {
        todo!()
    }

    pub fn parse_dyn(self, s: &str) -> Box<dyn Attribute> {
        todo!()
    }
}
