[![Build Status](https://travis-ci.org/PoiScript/iso8601-duration.svg?branch=master)](https://travis-ci.org/PoiScript/iso8601-duration)
[![docs.rs](https://docs.rs/iso8601-duration/badge.svg)](https://docs.rs/iso8601-duration)
[![crates.io](https://img.shields.io/crates/v/iso8601-duration.svg)](https://crates.io/crates/iso8601-duration)

# iso8601-duration

Parse ISO8601 duration format.

<https://en.wikipedia.org/wiki/ISO_8601#Durations>

## Installation

```toml
iso8601-duration = "0.1.0"
```

## Usage

```rust
use iso8601_duration::Duration;

 assert_eq!(
     "P3Y6M4DT12H30M5S".parse(),
     Ok(Duration::new(3., 6., 4., 12., 30., 5.))
 );
 assert_eq!("P23DT23H".parse::<Duration>().unwrap().num_hours(), Some(575.));
 assert_eq!("P0.5Y".parse::<Duration>().unwrap().num_years(), Some(0.5));
 assert_eq!("P0.5Y0.5M".parse::<Duration>().unwrap().num_months(), Some(6.5));
 assert_eq!("P12W".parse::<Duration>().unwrap().num_days(), Some(84.));

 assert!("PT".parse::<Duration>().is_err());
 assert!("P12WT12H30M5S".parse::<Duration>().is_err());
 assert!("P0.5S0.5M".parse::<Duration>().is_err());
 assert!("P0.5A".parse::<Duration>().is_err());
```

## `year` and `month`

`Duration` can be converted to either `std::time::Duration` or
`chrono::Duration` by calling `to_std` or `to_chrono`.

Both `to_std` and `to_chrono` will return `None` if the duration
includes `year` and `month`. Because ISO8601 duration format allows
the usage of `year` and `month`, and these durations are non-standard.
Since months can have 28, 29 30, 31 days, and years can have either
365 or 366 days.

To perform a lossless conversion, a starting date must be specified:

```rust
// requires `chrono` feature

use iso8601_duration::Duration;
use chrono::DateTime;

let one_month: Duration = "P1M".parse().unwrap();
let date = DateTime::parse_from_rfc3339("2000-02-01T00:00:00Z").unwrap();
assert_eq!(
    one_month.to_chrono_at_datetime(date).num_days(),
    29 // 2000 is a leap year
);
```

License: MIT
