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
            .and_then(|k| self.inner.get(*k))
    }
}
