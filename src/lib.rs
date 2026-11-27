//! **dhoni** converts Bengali text into a phonetic Roman counterpart.
//!
//! Each transliteration convention lives in its own self-contained module:
//!
//! - [`avro`] — Avro Phonetic-compatible spelling
//! - [`khipro`] — Khipro layout spelling
//!
//! The output is ASCII lowercase and aims to be compatible with common
//! phonetic schemes rather than a lossless round-trip.
//!
//! # Examples
//! ```rust
//! use dhoni::avro;
//! use dhoni::khipro;
//!
//! assert_eq!(avro::convert("আমি"), "ami");
//! assert_eq!(khipro::convert("চাঁদ"), "ca/d");
//! ```

pub mod avro;
pub mod khipro;

mod utils;
