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
//! use iso8601_duration::{ Duration, DurationParseError };
//! use std::time::Duration as StdDuration;
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
//!  assert_eq!(
//!      Duration::parse("PT30M5S").unwrap().to_std(),
//!      StdDuration::new(30 * 60 + 5, 0)
//!  );
//!  assert_eq!(
//!      Duration::parse("PT5H5M5S").unwrap().to_std(),
//!      StdDuration::new(5 * 3600 + 5 * 60 + 5, 0)
//!  );
//!  assert_eq!(
//!      Duration::parse("PT5H5M5.555S").unwrap().to_std(),
//!      StdDuration::new(5 * 3600 + 5 * 60 + 5, 555_000_000)
//!  );
//!  assert_eq!(
//!      Duration::parse("PT"),
//!      Err(DurationParseError::new("Parsing Error: Error { input: \"\", code: Verify }"))
//!  );
//!  assert_eq!(
//!      Duration::parse("P12WT12H30M5S"),
//!      Err(DurationParseError::new("Parsing Error: Error { input: \"T12H30M5S\", code: Eof }"))
//!  );
//!  assert_eq!(
//!      Duration::parse("P0.5S0.5M"),
//!      Err(DurationParseError::new("Parsing Error: Error { input: \"0.5S0.5M\", code: Verify }"))
//!  );
//!  assert_eq!(
//!      Duration::parse("P0.5A"),
//!      Err(DurationParseError::new("Parsing Error: Error { input: \"0.5A\", code: Verify }"))
//!  );
//! ```

mod duration;

pub use crate::duration::{Duration, DurationParseError};
