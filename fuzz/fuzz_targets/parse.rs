//! Fuzz target for the duration parser and the conversions after it.
//!
//! For any text, `parse` must not panic. For each parsed duration:
//! - every field is finite and not negative,
//! - `Display` output parses back to the same value,
//! - `to_std` and `to_chrono` return a value or `None` and do not panic.
//!
//! Curated seed inputs live in `fuzz/seeds/parse` and are tracked in git.
//! `fuzz/corpus/parse` holds generated inputs and is ignored. Run with:
//!
//! ```sh
//! cargo +nightly fuzz run parse fuzz/corpus/parse fuzz/seeds/parse
//! ```

#![no_main]

use iso8601_duration::Duration;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &str| {
    let Ok(duration) = input.parse::<Duration>() else {
        return;
    };

    check_fields(input, &duration);
    check_round_trip(input, &duration);

    let _ = duration.to_std();
    let _ = duration.to_chrono();
});

/// Asserts that every field of `duration` is finite and not negative.
fn check_fields(input: &str, duration: &Duration) {
    let fields = [
        duration.year,
        duration.month,
        duration.day,
        duration.hour,
        duration.minute,
        duration.second,
    ];

    assert!(
        fields
            .iter()
            .all(|field| field.is_finite() && *field >= 0.0),
        "{input:?} gave {duration:?}"
    );
}

/// Asserts that the `Display` text of `duration` parses to the same value.
fn check_round_trip(input: &str, duration: &Duration) {
    let printed = duration.to_string();
    let reparsed = printed.parse::<Duration>();

    assert_eq!(
        reparsed.as_ref(),
        Ok(duration),
        "{input:?} printed as {printed:?}"
    );
}
