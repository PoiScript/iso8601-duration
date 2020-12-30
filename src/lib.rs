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
//! use nom::{error::ErrorKind, error::Error};
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
//!      Err(nom::Err::Error(Error { input: "", code: ErrorKind::Verify }))
//!  );
//!  assert_eq!(
//!      Duration::parse("P12WT12H30M5S"),
//!      Err(nom::Err::Error(Error { input: "T12H30M5S", code: ErrorKind::Eof }))
//!  );
//!  assert_eq!(
//!      Duration::parse("P0.5S0.5M"),
//!      Err(nom::Err::Error(Error { input: "0.5S0.5M", code: ErrorKind::Verify }))
//!  );
//!  assert_eq!(
//!      Duration::parse("P0.5A"),
//!      Err(nom::Err::Error(Error { input: "0.5A", code: ErrorKind::Verify }))
//!  );
//! ```

mod duration;

pub use crate::duration::Duration;
