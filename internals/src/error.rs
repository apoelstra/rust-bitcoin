// SPDX-License-Identifier: CC0-1.0

//! # Error
//!
//! Error handling macros and helpers.
//!

use core::fmt;
use crate::InternalsToken;

pub mod input_string;
mod parse_error;

pub use input_string::InputString;

/// Token used to prevent error traits from being used directly
pub struct Token {
    _sealed: ()
}

/// Formats error.
///
/// If `std` feature is OFF appends error source (delimited by `: `). We do this because
/// `e.source()` is only available in std builds, without this macro the error source is lost for
/// no-std builds.
#[macro_export]
macro_rules! write_err {
    ($writer:expr, $string:literal $(, $args:expr)*; $source:expr) => {
        {
            #[cfg(feature = "std")]
            {
                let _ = &$source;   // Prevents clippy warnings.
                write!($writer, $string $(, $args)*)
            }
            #[cfg(not(feature = "std"))]
            {
                write!($writer, concat!($string, ": {}") $(, $args)*, $source)
            }
        }
    }
}

/// TODO doc, warning
#[cfg(feature = "std")]
pub type StdSource = dyn std::error::Error + 'static;

/// TODO doc, warning
#[cfg(not(feature = "std"))]
pub type StdSource = dyn fmt::Display + 'static;

/// This trait has a different signature depending on whethe

/// This trait has a different signature depending on whether `std` is enabled.
///
/// You should be very careful when using this to avoid breaking compilation
/// of your crate.
pub trait StdError {
    /// Formats the error type. Do not include any source information.
    fn fmt_without_source(&self, f: &mut fmt::Formatter, _: &InternalsToken) -> fmt::Result;

    /// Returns the source of the error. When std is on, this yields
    fn source(&self, _: &InternalsToken) -> Option<&StdSource>;

    /// Default method which formats the error type, choosing whether
    /// to include source information based on whether `std` is on.
    fn fmt(&self, f: &mut fmt::Formatter, tok: &InternalsToken) -> fmt::Result {
        StdError::fmt_without_source(self, f, tok)?;
        #[cfg(not(feature = "std"))]
        fmt_generic_source(self.source(tok), f, tok)?;
        Ok(())
    }
}

/// Computes the `source` method for the `std::error::Error` trait
#[cfg(feature = "std")]
pub fn fmt_std_source<T: std::error::Error>(err: &T, f: &mut fmt::Formatter, tok: &InternalsToken) -> fmt::Result {
    fmt_generic_source(err.source(), f, tok)
}

/// Computes the `source` method for the `std::error::Error` trait
pub fn fmt_generic_source<T: fmt::Display + ?Sized>(source: Option<&T>, f: &mut fmt::Formatter, _: &InternalsToken) -> fmt::Result {
    if let Some(src) = source {
        f.write_str(": ")?;
        src.fmt(f)
    } else {
        Ok(())
    }
}

