[![Build Status](https://travis-ci.org/PoiScript/iso8601-duration.svg?branch=master)](https://travis-ci.org/PoiScript/iso8601-duration)
[![docs.rs](https://docs.rs/iso8601-duration/badge.svg)](https://docs.rs/iso8601-duration)
[![crates.io](https://img.shields.io/crates/v/iso8601-duration.svg)](https://crates.io/crates/iso8601-duration)

# iso8601-duration

Parse ISO8601 duration format.

## Installation

```toml
iso8601-duration = "0.1.0"
```

## Usage

```rust
use iso8601_duration::Duration;
use nom::{error::ErrorKind, Err};

 assert_eq!(
     Duration::parse("P23DT23H"),
     Ok(Duration::new(0., 0., 23., 23., 0., 0.))
 );
 assert_eq!(
     Duration::parse("P3Y6M4DT12H30M5S"),
     Ok(Duration::new(3., 6., 4., 12., 30., 5.))
 );
 assert_eq!(
     Duration::parse("P0.5Y"),
     Ok(Duration::new(0.5, 0., 0., 0., 0., 0.))
 );
 assert_eq!(
     Duration::parse("P0.5Y0.5M"),
     Ok(Duration::new(0.5, 0.5, 0., 0., 0., 0.))
 );
 assert_eq!(
     Duration::parse("P12W"),
     Ok(Duration::new(0., 0., 84., 0., 0., 0.))
 );

 assert_eq!(
     Duration::parse("PT"),
     Err(Err::Error(("", ErrorKind::Verify)))
 );
 assert_eq!(
     Duration::parse("P12WT12H30M5S"),
     Err(Err::Error(("T12H30M5S", ErrorKind::Eof)))
 );
 assert_eq!(
     Duration::parse("P0.5S0.5M"),
     Err(Err::Error(("0.5S0.5M", ErrorKind::Verify)))
 );
 assert_eq!(
     Duration::parse("P0.5A"),
     Err(Err::Error(("0.5A", ErrorKind::Verify)))
 );
```

## Changelog

### Unreleased

- Breaking: Updated nom to version 7

## License

MIT
