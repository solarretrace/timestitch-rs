////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Event record entries.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::TimeInterval;

// External library imports.
use anyhow::Error;
use serde::Deserialize;
use serde::Serialize;
use table_gen_core::Row;
use table_gen_core::Cell;

// Standard library imports.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fmt::Debug;



////////////////////////////////////////////////////////////////////////////////
// Attribute
////////////////////////////////////////////////////////////////////////////////
/// A record attribute.
pub trait Attribute: Cell + Debug {}

// Blanket impl for all Debug + Cell + 'static.
impl<T: Debug + Cell + 'static> Attribute for T {}


////////////////////////////////////////////////////////////////////////////////
// Entry
////////////////////////////////////////////////////////////////////////////////
/// The `Entry` record identifier type.
pub type EntryId = u64;

/// A indivisible record entry.
#[derive(Debug)]
pub struct Entry {
    /// The entries unique identifier.
    pub id: EntryId,
    /// The time interval over which the entry is valid.
    pub time: TimeInterval,
    /// The file source of the entry.
    
    pub source_path: DataSource,
    /// The reference source of the entry.
    pub source_ref: Box<str>,
    /// The entry contents.
    pub attributes: BTreeMap<Box<str>, Box<dyn Attribute + 'static>>,
}


impl Row for Entry {
    fn len(&self) -> usize {
        4 + self.attributes.len()
    }

    fn cell(&self, col_idx: usize) -> Option<&dyn Cell> {
        match col_idx {
            0 => Some(&self.id),
            1 => Some(&1000),
            2 => Some(&self.source_path),
            3 => Some(&self.source_ref),
            n =>  n.checked_sub(4)
                .and_then(|n| self.attributes
                    .values()
                    .nth(n)
                    .map(|a| a.as_ref() as &dyn Cell)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Input data source matching
////////////////////////////////////////////////////////////////////////////////
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

impl Default for MatchSource {
    fn default() -> Self { MatchSource::Default }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum MatchSourceAttribute {
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
    pub fn parse_dyn(self, s: &str) -> Result<Box<dyn Attribute>, Error> {
        use MatchFormat::*;
        match self {
            U64 => Ok(Box::new(s.parse::<u64>()?)),
            Time => todo!(),
            Text => Ok(Box::new(s.to_string().into_boxed_str())),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Input data source matching
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSource(pub PathBuf);

impl std::fmt::Display for DataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0.display(), f)
    }
}

impl PartialOrd for DataSource {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.as_os_str().partial_cmp(other.0.as_os_str())
    }
}
    
