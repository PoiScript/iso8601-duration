use std::time::Duration as StdDuration;
use std::{fmt, str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map_res, opt},
    error::{ErrorKind, ParseError},
    number::complete::float,
    sequence::{preceded, separated_pair, terminated, tuple},
    Err, Finish, IResult,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Duration {
    pub year: f32,
    pub month: f32,
    pub day: f32,
    pub hour: f32,
    pub minute: f32,
    pub second: f32,
}

impl Duration {
    pub fn new(year: f32, month: f32, day: f32, hour: f32, minute: f32, second: f32) -> Self {
        Duration {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    pub fn num_years(&self) -> Option<f32> {
        if self.second > 0.0 || self.minute > 0.0 || self.hour > 0.0 {
            return None;
        }

        Some(self.year + self.month / 12.)
    }

    pub fn num_months(&self) -> Option<f32> {
        if self.second > 0.0 || self.minute > 0.0 || self.hour > 0.0 {
            return None;
        }

        Some(self.year * 12. + self.month)
    }

    pub fn num_weeks(&self) -> Option<f32> {
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

    pub fn num_days(&self) -> Option<f32> {
        if self.month > 0.0 || self.year > 0.0 {
            return None;
        }

        Some(self.second / 60. / 60. / 24. + self.minute / 60. / 24. + self.hour / 24. + self.day)
    }

    pub fn num_hours(&self) -> Option<f32> {
        if self.month > 0.0 || self.year > 0.0 {
            return None;
        }

        Some(self.second / 60. / 60. + self.minute / 60. + self.hour + self.day * 24.)
    }

    pub fn num_minutes(&self) -> Option<f32> {
        if self.month > 0.0 || self.year > 0.0 {
            return None;
        }

        Some(self.second / 60. + self.minute + self.hour * 60. + self.day * 60. * 24.)
    }

    pub fn num_seconds(&self) -> Option<f32> {
        if self.month > 0.0 || self.year > 0.0 {
            return None;
        }

        Some(self.second + self.minute * 60. + self.hour * 60. * 60. + self.day * 60. * 60. * 24.)
    }

    pub fn to_std(&self) -> Option<StdDuration> {
        self.num_seconds().map(StdDuration::from_secs_f32)
    }

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

fn decimal_comma_number(input: &str) -> IResult<&str, f32> {
    map_res(separated_pair(digit1, tag(","), digit1), |(a, b)| {
        f32::from_str(&format!("{}.{}", a, b))
    })(input)
}

fn value_with_designator(designator: &str) -> impl Fn(&str) -> IResult<&str, f32> + '_ {
    move |input| {
        terminated(
            alt((float, decimal_comma_number, map_res(digit1, f32::from_str))),
            tag(designator),
        )(input)
    }
}

fn parse_basic_format(input: &str) -> IResult<&str, Duration> {
    let (input, (year, month, day)) = tuple((
        opt(value_with_designator("Y")),
        opt(value_with_designator("M")),
        opt(value_with_designator("D")),
    ))(input)?;

    let (input, time) = opt(preceded(
        tag("T"),
        tuple((
            opt(value_with_designator("H")),
            opt(value_with_designator("M")),
            opt(value_with_designator("S")),
        )),
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
