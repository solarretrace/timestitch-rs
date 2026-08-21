////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Utilities module.
////////////////////////////////////////////////////////////////////////////////

/// External library imports.
use regex::Captures;
use regex::Match;

// Standard library imports.
use std::collections::HashMap;


////////////////////////////////////////////////////////////////////////////////
// CapturesRemap
////////////////////////////////////////////////////////////////////////////////
/// A wrapper around regex `Captures` to reassign capture group indices.
#[derive(Debug)]
pub (in crate) struct CapturesMap<'a, 'b> {
    inner: Captures<'a>,
    map: &'b HashMap<usize, usize>,
}

impl<'a, 'b> CapturesMap<'a, 'b> {
    /// Wraps the given `Captures` with the provided index map.
    pub (in crate) fn new(inner: Captures<'a>, map: &'b HashMap<usize, usize>)
        -> Self
    {
        Self {
            inner,
            map,
        }
    }

    /// Returns the capture group at the given re-mapped index.
    pub (in crate) fn get(&self, idx: usize) -> Option<Match<'a>> {
        self.map.get(&idx)
            .or(Some(&idx))
            .and_then(|k| self.inner.get(*k))
    }
}



////////////////////////////////////////////////////////////////////////////////
// Serialization helpers
////////////////////////////////////////////////////////////////////////////////
/// Serde serialization/deserialization module for `Vec<Option<Regex>>`
pub (in crate) mod vec_option_regex {
    use regex::Regex;
    use serde::Deserialize as _;
    use serde::Deserializer;
    use serde::ser::SerializeSeq;
    use serde::Serializer;
    use serde_regex::Serde;

    /// Serializes a `Vec<Option<Regex>>` value into `s`.
    #[allow(unused)]
    pub (in crate) fn serialize<S: Serializer>(v: &Vec<Option<Regex>>, s: S)
        -> Result<S::Ok, S::Error>
    {
        let mut seq = s.serialize_seq(Some(v.len()))?;
        for item in v {
            // Uses Serde<&Option<Regex>> impl
            seq.serialize_element(&Serde(item))?;
        }
        seq.end()
    }

    /// Deserializes a `Vec<Option<Regex>>` value from `d`.
    #[allow(unused)]
    pub (in crate) fn deserialize<'de, D: Deserializer<'de>>(d: D)
        -> Result<Vec<Option<Regex>>, D::Error>
    {
        let wrapped: Vec<Serde<Option<Regex>>> = Vec::deserialize(d)?;
        Ok(wrapped.into_iter().map(Serde::into_inner).collect())
    }
}
