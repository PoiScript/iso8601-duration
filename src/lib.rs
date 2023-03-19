//! Parse ISO8601 duration format.
//!
//! # Installation
//!
//! ```toml
//! iso8601-duration = "0.1.0"
//! ```
//!
//! # Usage
//!
//! ```rust
//! use iso8601_duration::Duration;
//!
//!  assert_eq!(
//!      Duration::parse("P23DT23H"),
//!      Ok(Duration::new(0., 0., 23., 23., 0., 0.))
//!  );
//!  assert_eq!(
//!      Duration::parse("P3Y6M4DT12H30M5S"),
//!      Ok(Duration::new(3., 6., 4., 12., 30., 5.))
//!  );
//!  assert_eq!(
//!      Duration::parse("P0.5Y"),
//!      Ok(Duration::new(0.5, 0., 0., 0., 0., 0.))
//!  );
//!  assert_eq!(
//!      Duration::parse("P0.5Y0.5M"),
//!      Ok(Duration::new(0.5, 0.5, 0., 0., 0., 0.))
//!  );
//!  assert_eq!(
//!      Duration::parse("P12W"),
//!      Ok(Duration::new(0., 0., 84., 0., 0., 0.))
//!  );
//!
//!  assert!(Duration::parse("PT").is_err());
//!  assert!(Duration::parse("P12WT12H30M5S").is_err());
//!  assert!(Duration::parse("P0.5S0.5M").is_err());
//!  assert!(Duration::parse("P0.5A").is_err());
//! ```

mod duration;
#[cfg(feature = "serde")]
mod serde;

pub use crate::duration::{Duration, ParseDurationError};
