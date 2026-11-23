use std::time::Duration as StdDuration;
use std::{fmt, str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    character::complete::one_of,
    combinator::{all_consuming, map_opt, opt, recognize, verify},
    error::{ErrorKind, ParseError},
    sequence::{pair, preceded, terminated, tuple},
    Err, Finish, IResult,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Duration {
    pub year: f64,
    pub month: f64,
    pub day: f64,
    pub hour: f64,
    pub minute: f64,
    pub second: f64,
}

impl Duration {
    /// Create a new duration
    pub fn new(year: f64, month: f64, day: f64, hour: f64, minute: f64, second: f64) -> Self {
        Duration {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    /// Return number of years in the duration
    ///
    /// This method will return `None` if `Duration` contains
    /// `second`, `minute` or `hour`.
    pub fn num_years(&self) -> Option<f64> {
        if self.second > 0.0 || self.minute > 0.0 || self.hour > 0.0 {
            return None;
        }

        Some(self.year + self.month / 12.)
    }

    /// Return number of months in the duration
    ///
    /// This method will return `None` if `Duration` contains
    /// `second`, `minute` or `hour`.
    pub fn num_months(&self) -> Option<f64> {
        if self.second > 0.0 || self.minute > 0.0 || self.hour > 0.0 {
            return None;
        }

        Some(self.year * 12. + self.month)
    }

    /// Return number of weeks in the duration
    ///
    /// This method will return `None` if `Duration` contains
    /// `year` or `month`.
    pub fn num_weeks(&self) -> Option<f64> {
        if self.month > 0.0 || self.year > 0.0 {
            return None;
        }

        Some(
            self.second / 60. / 60. / 24. / 7.
                + self.minute / 60. / 24. / 7.
                + self.hour / 24. / 7.
                + self.day / 7.,
        )
    }

    /// Return number of days in the duration
    ///
    /// This method will return `None` if `Duration` contains
    /// `year` or `month`.
    pub fn num_days(&self) -> Option<f64> {
        if self.month > 0.0 || self.year > 0.0 {
            return None;
        }

        Some(self.second / 60. / 60. / 24. + self.minute / 60. / 24. + self.hour / 24. + self.day)
    }

    /// Return number of hours in the duration
    ///
    /// This method will return `None` if `Duration` contains
    /// `year` or `month`.
    pub fn num_hours(&self) -> Option<f64> {
        if self.month > 0.0 || self.year > 0.0 {
            return None;
        }

        Some(self.second / 60. / 60. + self.minute / 60. + self.hour + self.day * 24.)
    }

    /// Return number of minutes in the duration
    ///
    /// This method will return `None` if `Duration` contains
    /// `year` or `month`.
    pub fn num_minutes(&self) -> Option<f64> {
        if self.month > 0.0 || self.year > 0.0 {
            return None;
        }

        Some(self.second / 60. + self.minute + self.hour * 60. + self.day * 60. * 24.)
    }

    /// Return number of seconds in the duration
    ///
    /// This method will return `None` if `Duration` contains
    /// `year` or `month`.
    pub fn num_seconds(&self) -> Option<f64> {
        if self.month > 0.0 || self.year > 0.0 {
            return None;
        }

        Some(self.second + self.minute * 60. + self.hour * 60. * 60. + self.day * 60. * 60. * 24.)
    }

    /// Convert duration to `std::time::Duration`.
    ///
    /// This method returns `None` if the `Duration` contains `year` or
    /// `month`, if any component is negative or not finite, or if the result
    /// is too large for `std::time::Duration`. Each component is rounded to
    /// the nearest nanosecond on its own, and the results are added.
    ///
    /// To convert a `Duration` that contains `year` or `month`, use
    /// `to_chrono_at_datetime` (requires the `chrono` feature). See the
    /// [crate documentation](crate#year-and-month).
    pub fn to_std(&self) -> Option<StdDuration> {
        if self.month > 0.0 || self.year > 0.0 {
            return None;
        }

        // Convert each component on its own, then add exact integer durations.
        // One f64 sum of all components loses nanoseconds on long durations.
        self.seconds_by_component()
            .iter()
            .try_fold(StdDuration::ZERO, |total, &seconds| {
                total.checked_add(StdDuration::try_from_secs_f64(seconds).ok()?)
            })
    }

    /// Returns the day, hour, minute and second components, each in seconds.
    pub(crate) fn seconds_by_component(&self) -> [f64; 4] {
        [
            self.day * 60. * 60. * 24.,
            self.hour * 60. * 60.,
            self.minute * 60.,
            self.second,
        ]
    }

    /// Parse given string into Duration
    pub fn parse(input: &str) -> Result<Duration, ParseDurationError> {
        all_consuming(preceded(
            tag("P"),
            alt((parse_week_format, parse_basic_format)),
        ))(input)
        .finish()
        .map(|(_, duration)| duration)
        .map_err(|err| ParseDurationError::new(input, err))
    }
}

#[derive(PartialEq, Eq)]
pub struct ParseDurationError {
    pub input: String,
    pub position: usize,
    pub kind: ErrorKind,
}

impl ParseDurationError {
    fn new(input: &str, err: nom::error::Error<&str>) -> Self {
        ParseDurationError {
            input: input.to_string(),
            position: input.len() - err.input.len(),
            kind: err.code,
        }
    }
}

impl fmt::Debug for ParseDurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error: {:?} in {:?} at position {}",
            self.kind, self.input, self.position
        )
    }
}

impl FromStr for Duration {
    type Err = ParseDurationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Duration::parse(s)
    }
}

impl fmt::Display for Duration {
    /// Writes the duration in the ISO 8601 basic format.
    ///
    /// Only positive components are written. A duration with no positive
    /// component is written as `PT0S`, which parses again.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = [
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        ];

        if !fields.iter().any(|&field| field > 0.0) {
            return f.write_str("PT0S");
        }

        f.write_str("P")?;
        if self.year > 0.0 {
            write!(f, "{}Y", self.year)?;
        }
        if self.month > 0.0 {
            write!(f, "{}M", self.month)?;
        }
        if self.day > 0.0 {
            write!(f, "{}D", self.day)?;
        }
        if self.hour > 0.0 || self.minute > 0.0 || self.second > 0.0 {
            f.write_str("T")?;
        }
        if self.hour > 0.0 {
            write!(f, "{}H", self.hour)?;
        }
        if self.minute > 0.0 {
            write!(f, "{}M", self.minute)?;
        }
        if self.second > 0.0 {
            write!(f, "{}S", self.second)?;
        }
        Ok(())
    }
}

/// Parses a non-negative decimal number with `.` or `,` as the decimal sign.
///
/// ISO 8601 allows digits with an optional fraction only. Signs, exponents,
/// `inf` and `NaN` are rejected. For compatibility with 0.2, a fraction
/// without an integer part (`.5`) is accepted. A value that overflows `f64`
/// is an error.
fn decimal_number(input: &str) -> IResult<&str, f64> {
    let decimal_sign = || one_of(".,");
    let text = alt((
        recognize(pair(digit1, opt(pair(decimal_sign(), digit1)))),
        recognize(pair(decimal_sign(), digit1)),
    ));

    map_opt(text, |text: &str| {
        f64::from_str(&text.replace(',', "."))
            .ok()
            .filter(|value| value.is_finite())
    })(input)
}

/// Parses a decimal number that the given designator letter follows.
fn value_with_designator(designator: &str) -> impl Fn(&str) -> IResult<&str, f64> + '_ {
    move |input| terminated(decimal_number, tag(designator))(input)
}

fn parse_basic_format(input: &str) -> IResult<&str, Duration> {
    let (input, (year, month, day)) = tuple((
        opt(value_with_designator("Y")),
        opt(value_with_designator("M")),
        opt(value_with_designator("D")),
    ))(input)?;

    // A `T` designator must have at least one time component after it.
    let (input, time) = opt(preceded(
        tag("T"),
        verify(
            tuple((
                opt(value_with_designator("H")),
                opt(value_with_designator("M")),
                opt(value_with_designator("S")),
            )),
            |(hour, minute, second)| hour.is_some() || minute.is_some() || second.is_some(),
        ),
    ))(input)?;

    let (hour, minute, second) = time.unwrap_or_default();

    if year.is_none()
        && month.is_none()
        && day.is_none()
        && hour.is_none()
        && minute.is_none()
        && second.is_none()
    {
        Err(Err::Error(ParseError::from_error_kind(
            input,
            ErrorKind::Verify,
        )))
    } else {
        Ok((
            input,
            Duration {
                year: year.unwrap_or_default(),
                month: month.unwrap_or_default(),
                day: day.unwrap_or_default(),
                hour: hour.unwrap_or_default(),
                minute: minute.unwrap_or_default(),
                second: second.unwrap_or_default(),
            },
        ))
    }
}

fn parse_week_format(input: &str) -> IResult<&str, Duration> {
    let (input, week) = value_with_designator("W")(input)?;

    Ok((
        input,
        Duration {
            year: 0.,
            month: 0.,
            day: week * 7.,
            hour: 0.,
            minute: 0.,
            second: 0.,
        },
    ))
}

fn _parse_extended_format(_input: &str) -> IResult<&str, Duration> {
    unimplemented!()
}
