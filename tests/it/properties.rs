//! Property tests for parsing, printing and conversion.
//!
//! The precision properties compare against an oracle in integer
//! nanoseconds. The oracle uses the generated digits, never a float.

use iso8601_duration::Duration;
use proptest::prelude::*;

/// Nanoseconds in one second.
const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// One duration component: an integer part and optional fraction digits.
#[derive(Debug, Clone)]
struct Component {
    /// The digits before the decimal sign.
    integer: u32,
    /// The digits after the decimal sign. `None` writes no decimal sign.
    fraction: Option<String>,
    /// Write a comma as the decimal sign, not a full stop.
    comma: bool,
}

impl Component {
    /// Writes the component as ISO 8601 text, without the designator.
    fn text(&self) -> String {
        match &self.fraction {
            Some(digits) => {
                let sign = if self.comma { ',' } else { '.' };

                format!("{}{sign}{digits}", self.integer)
            }
            None => self.integer.to_string(),
        }
    }
}

/// A component with an integer part below `max` and at most 9 fraction digits.
fn component(max: u32) -> impl Strategy<Value = Component> {
    (0..max, proptest::option::of("[0-9]{1,9}"), any::<bool>()).prop_map(
        |(integer, fraction, comma)| Component {
            integer,
            fraction,
            comma,
        },
    )
}

/// Any valid duration in the basic format, built from optional components.
fn basic_format() -> impl Strategy<Value = String> {
    let date = proptest::collection::vec(proptest::option::of(component(100_000)), 3);
    let time = proptest::collection::vec(proptest::option::of(component(100_000)), 3);

    (date, time)
        .prop_filter("at least one component", |(date, time)| {
            date.iter().chain(time).any(Option::is_some)
        })
        .prop_map(|(date, time)| {
            let mut text = String::from("P");
            append_components(&mut text, &date, ['Y', 'M', 'D']);

            if time.iter().any(Option::is_some) {
                text.push('T');
                append_components(&mut text, &time, ['H', 'M', 'S']);
            }

            text
        })
}

/// Text that starts with `P` and mixes number fragments with designators.
///
/// The tokens include signs, exponents and special values, so that the
/// parser sees many texts that are almost valid.
fn near_valid_text() -> impl Strategy<Value = String> {
    let token = prop_oneof![
        "[0-9]{1,3}",
        "9{39,320}",
        Just(".".to_owned()),
        Just(",".to_owned()),
        Just("-".to_owned()),
        Just("+".to_owned()),
        Just("e".to_owned()),
        Just("inf".to_owned()),
        Just("NaN".to_owned()),
        "[YMWDTHS]",
    ];

    proptest::collection::vec(token, 0..10).prop_map(|tokens| format!("P{}", tokens.concat()))
}

/// Appends each present component with its designator.
fn append_components(text: &mut String, components: &[Option<Component>], designators: [char; 3]) {
    for (component, designator) in components.iter().zip(designators) {
        if let Some(component) = component {
            text.push_str(&component.text());
            text.push(designator);
        }
    }
}

/// Converts fraction digits to nanoseconds. Digits after the ninth are ignored.
fn fraction_nanos(digits: &str) -> u128 {
    format!("{digits:0<9}")[..9]
        .parse()
        .expect("nine ASCII digits always parse as u128")
}

proptest! {
    /// The parser returns `Ok` or `Err` for any text, and never panics.
    #[test]
    fn parse_never_panics(input in "\\PC*") {
        let _ = input.parse::<Duration>();
    }

    /// A parsed duration has finite, non-negative fields.
    /// Its conversions return a value or `None`, and never panic.
    #[test]
    fn parsed_values_are_well_formed(input in near_valid_text()) {
        if let Ok(duration) = input.parse::<Duration>() {
            for field in [
                duration.year, duration.month, duration.day,
                duration.hour, duration.minute, duration.second,
            ] {
                prop_assert!(field.is_finite() && field >= 0.0, "{input:?} gave {duration:?}");
            }

            let _ = duration.to_std();
            #[cfg(feature = "chrono")]
            let _ = duration.to_chrono();
        }
    }

    /// Every valid basic-format text parses.
    #[test]
    fn valid_text_parses(input in basic_format()) {
        prop_assert!(input.parse::<Duration>().is_ok(), "{input:?} must parse");
    }

    /// Printing a parsed duration and parsing the text again gives the same value.
    #[test]
    fn display_round_trips(input in basic_format()) {
        let duration: Duration = input.parse().expect("basic_format gives valid text");
        let printed = duration.to_string();
        let reparsed: Duration = printed
            .parse()
            .map_err(|err| TestCaseError::fail(format!("{printed:?}: {err:?}")))?;

        prop_assert_eq!(reparsed, duration, "{:?} printed as {:?}", input, printed);
    }

    /// `to_std` is exact to the nanosecond for durations up to 4 days.
    ///
    /// The total stays below 2^53 ns, so `f64` holds it with a margin.
    #[test]
    fn to_std_is_exact_for_short_durations(
        days in 0u32..4,
        hours in 0u32..24,
        minutes in 0u32..60,
        seconds in 0u32..60,
        fraction in "[0-9]{1,9}",
    ) {
        let input = format!("P{days}DT{hours}H{minutes}M{seconds}.{fraction}S");
        let whole = ((u128::from(days) * 24 + u128::from(hours)) * 60 + u128::from(minutes)) * 60
            + u128::from(seconds);
        let expected = whole * NANOS_PER_SECOND + fraction_nanos(&fraction);
        let actual = input.parse::<Duration>().expect("valid text").to_std();

        prop_assert_eq!(actual.map(|d| d.as_nanos()), Some(expected), "{}", input);
    }

    /// `to_std` is exact to the nanosecond for durations up to about 27 years.
    ///
    /// Each component converts on its own, so a long day count does not
    /// change the millisecond part. One f64 sum of all components gives
    /// an error of about 100 ns here. The original f32 bug lost whole
    /// milliseconds.
    #[test]
    fn to_std_keeps_milliseconds_for_long_durations(
        days in 0u32..10_000,
        hours in 0u32..24,
        millis in 0u32..60_000,
    ) {
        let input = format!("P{days}DT{hours}H{}.{:03}S", millis / 1000, millis % 1000);
        let expected = (u128::from(days) * 24 + u128::from(hours)) * 3600 * NANOS_PER_SECOND
            + u128::from(millis) * 1_000_000;
        let actual = input.parse::<Duration>().expect("valid text").to_std().expect("in range");

        prop_assert_eq!(actual.as_nanos(), expected, "{}", input);
    }
}

/// Precision properties for the `chrono` conversion.
#[cfg(feature = "chrono")]
mod chrono {
    use iso8601_duration::Duration;
    use proptest::prelude::*;

    proptest! {
        /// `to_chrono` matches `to_std` to the nanosecond for durations up to 4 days.
        #[test]
        fn to_chrono_matches_to_std(
            days in 0u32..4,
            hours in 0u32..24,
            seconds in 0u32..3600,
            fraction in "[0-9]{1,9}",
        ) {
            let input = format!("P{days}DT{hours}H{seconds}.{fraction}S");
            let duration: Duration = input.parse().expect("valid text");
            let std = duration.to_std().expect("in range");
            let chrono = duration.to_chrono().expect("in range");

            prop_assert_eq!(chrono.to_std().ok(), Some(std), "{}", input);
        }
    }
}
