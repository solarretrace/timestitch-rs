////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Record attributes module.
////////////////////////////////////////////////////////////////////////////////

// Standard library imports.
use std::fmt::Display;
use std::fmt::Debug;
use std::cmp::Ordering;
use std::any::Any;


////////////////////////////////////////////////////////////////////////////////
// Attribute
////////////////////////////////////////////////////////////////////////////////
/// A recorded attribute.
pub trait Attribute: Debug + Display {
    /// Convert a cell to `Any` to enable downcasting to the base type.
    fn as_any(&self) -> &dyn Any;

    /// Perform a partial compare to another attribute. This will return `None`
    /// when comparing attributes of different types.
    fn dyn_partial_cmp(&self, other: &dyn Attribute) -> Option<Ordering>;
}

// Blanket impl for all PartialOrd + Display + 'static.
impl<T: Debug + Display + PartialOrd + 'static> Attribute for T {
    fn as_any(&self) -> &dyn Any { self }

    fn dyn_partial_cmp(&self, other: &dyn Attribute) -> Option<Ordering> {
        other.as_any()
            .downcast_ref::<T>()
            .and_then(|o| self.partial_cmp(o))
    }
}
