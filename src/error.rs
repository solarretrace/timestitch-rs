////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Common error types.
////////////////////////////////////////////////////////////////////////////////

// Standard library imports.
use std::fmt::Display;
use std::fmt::Debug;


////////////////////////////////////////////////////////////////////////////////
// CaptureGroupCountError
////////////////////////////////////////////////////////////////////////////////
/// An incorrect number of capture groups have been provided for the calendar
/// parser.
#[derive(Debug, Clone, Copy)]
pub struct CaptureGroupCountError<F> {
    /// The format.
    pub format: F,
    /// The minimum number of capture groups to use.
    pub min: u8,
    /// The maximum number of capture groups to use.
    pub max: u8,
}

impl<F> Display for CaptureGroupCountError<F>
    where F: Display 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.min == self.max {
            write!(f, "invalid capture group count for format '{}': \
                expected {} groups", self.format, self.min)
        } else {
            write!(f, "invalid capture group count for format '{}': \
                expected {} to {} groups", self.format, self.min, self.max)
        }
    }
}

impl<F> std::error::Error for CaptureGroupCountError<F>
    where F: Display + Debug
{}
