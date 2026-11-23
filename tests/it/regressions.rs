//! Fixed inputs for parser and conversion bugs that were found.

use std::time::Duration as StdDuration;

use iso8601_duration::Duration;

/// Parses `input` and panics with the parse error if the input is not valid.
fn parse(input: &str) -> Duration {
    input
        .parse()
        .unwrap_or_else(|err| panic!("{:?} must parse: {:?}", input, err))
}

/// A fraction of a second keeps its exact value in `to_std`.
///
/// Before the fix, the f32 sum gave 1.001000047 s.
#[test]
fn fractional_seconds_are_exact() {
    assert_eq!(
        parse("PT1.001S").to_std(),
        Some(StdDuration::from_millis(1001))
    );
}

/// Milliseconds stay exact when the duration is longer than 4.5 hours.
///
/// Before the fix, the f32 sum gave 18000.001953125 s.
#[test]
fn long_durations_keep_milliseconds() {
    assert_eq!(
        parse("PT5H0.001S").to_std(),
        Some(StdDuration::from_millis(5 * 3_600_000 + 1))
    );
    assert_eq!(
        parse("P400DT0.001S").to_std(),
        Some(StdDuration::from_millis(400 * 86_400_000 + 1))
    );
}

/// The parser accepts a comma as the decimal sign, as ISO 8601 requires.
#[test]
fn comma_decimal_sign_is_accepted() {
    assert_eq!(parse("P1,5D"), parse("P1.5D"));
    assert_eq!(parse("PT0,25S"), parse("PT0.25S"));
}

/// A fraction without an integer part is accepted for compatibility with 0.2.
#[test]
fn leading_decimal_sign_is_accepted() {
    assert_eq!(parse("PT.5S"), parse("PT0.5S"));
}

/// The parser rejects number forms that ISO 8601 does not define.
///
/// Before the fix, nom's `float` accepted signs, exponents and `inf`.
/// Negative and infinite values then made `to_std` panic.
#[test]
fn non_iso_numbers_are_rejected() {
    for input in [
        "P-1D", "PT+1S", "PT1e3S", "PT1E3S", "PT1e40S", "PinfD", "PnanS", "PT1.S", "PT1,S",
    ] {
        assert!(
            input.parse::<Duration>().is_err(),
            "{:?} must be rejected",
            input
        );
    }
}

/// A value too large for `f64` is an error, not an infinite field.
#[test]
fn overflowing_values_are_rejected() {
    let input = format!("PT{}S", "9".repeat(400));

    assert!(input.parse::<Duration>().is_err());
}

/// A `T` designator must have at least one time component after it.
#[test]
fn empty_time_part_is_rejected() {
    assert!("P1DT".parse::<Duration>().is_err());
    assert!("PT".parse::<Duration>().is_err());
}

/// The zero duration prints as `PT0S`, which parses again.
///
/// Before the fix, it printed as `P`, which is not a valid duration.
#[test]
fn zero_duration_round_trips() {
    let zero = parse("PT0S");

    assert_eq!(zero.to_string(), "PT0S");
    assert_eq!(parse(&zero.to_string()), zero);
    assert_eq!(parse("P0D").to_string(), "PT0S");
}

/// `to_std` returns `None` for values it cannot represent, and does not panic.
#[test]
fn to_std_rejects_unrepresentable_values() {
    assert_eq!(Duration::new(0., 0., -1., 0., 0., 0.).to_std(), None);
    assert_eq!(
        Duration::new(0., 0., 0., 0., 0., f64::INFINITY).to_std(),
        None
    );
    assert_eq!(Duration::new(0., 0., 0., 0., 0., f64::NAN).to_std(), None);
    assert_eq!(Duration::new(0., 0., 1e300, 0., 0., 0.).to_std(), None);
}

/// Regressions for the `chrono` conversions.
#[cfg(feature = "chrono")]
mod chrono {
    use chrono::TimeDelta;

    use super::parse;
    use iso8601_duration::Duration;

    /// A fraction of a second keeps its exact value in `to_chrono`.
    ///
    /// Before the fix, the fraction was truncated, not rounded.
    #[test]
    fn fractional_seconds_are_exact() {
        assert_eq!(
            parse("PT1.001S").to_chrono(),
            Some(TimeDelta::milliseconds(1001))
        );
        assert_eq!(
            parse("PT5H0.001S").to_chrono(),
            Some(TimeDelta::milliseconds(5 * 3_600_000 + 1))
        );
        assert_eq!(
            parse("P400DT0.001S").to_chrono(),
            Some(TimeDelta::milliseconds(400 * 86_400_000 + 1))
        );
    }

    /// `to_chrono` returns `None` for values it cannot represent, and does not panic.
    #[test]
    fn to_chrono_rejects_unrepresentable_values() {
        assert_eq!(Duration::new(0., 0., 1e300, 0., 0., 0.).to_chrono(), None);
        assert_eq!(
            Duration::new(0., 0., 0., 0., 0., f64::INFINITY).to_chrono(),
            None
        );
        assert_eq!(
            Duration::new(0., 0., 0., 0., 0., f64::NAN).to_chrono(),
            None
        );
    }
}
