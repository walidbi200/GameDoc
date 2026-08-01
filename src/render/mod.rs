//! The `render` module contains independent rendering back-ends.
//!
//! Each sub-module converts a [`crate::model::Report`] into a particular
//! output format.  The rendering layer has **no knowledge of the Linux
//! system** — it only operates on the data structures defined in
//! [`crate::model`].

pub mod color;
pub mod json;
pub mod markdown;
pub mod terminal;
