use std::fmt;
use std::str::FromStr;
use std::time::Duration as StdDuration;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map_res, opt},
    error::{Error, ErrorKind, ParseError},
    number::complete::float,
    sequence::{preceded, separated_pair, terminated, tuple},
    Err, IResult,
};

const YEAR_IN_S: f64 = 31556952.0; // gregorian - includes leap-seconds

#[derive(Debug, PartialEq)]
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

    // In this scheme we try to balance the flexibility of fractional units
    // with the need to avoid rounding errors caused by floating point drift.
    // The smaller we keep the floats, the better.
    pub fn to_std(&self) -> StdDuration {
        let millis = (self.second.fract() * 1000.0).round() as u64;
        StdDuration::from_millis(
            ((self.year as f64 * YEAR_IN_S).round() as u64
                + (self.month * 30.42 * 60.0 * 60.0 * 24.0).round() as u64
                + (self.day * 24.0 * 60.0 * 60.0).round() as u64
                + (self.hour * 60.0 * 60.0).round() as u64
                + (self.minute * 60.0).round() as u64
                + self.second.trunc() as u64)
                * 1000
                + millis,
        )
    }

    pub fn parse(input: &str) -> Result<Duration, DurationParseError> {
        let (_, duration) = all_consuming(preceded(
            tag("P"),
            alt((parse_week_format, parse_basic_format)),
        ))(input)?;

        Ok(duration)
    }
}

impl FromStr for Duration {
    type Err = DurationParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Duration::parse(s).map_err(DurationParseError::from)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationParseError(String);

impl DurationParseError {
    pub fn new<S: Into<String>>(s: S) -> DurationParseError {
        DurationParseError(s.into())
    }
}

impl From<Err<Error<&str>>> for DurationParseError {
    fn from(err: Err<Error<&str>>) -> Self {
        DurationParseError(err.to_string())
    }
}

impl fmt::Display for DurationParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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
            alt((
                float,
                decimal_comma_number,
                map_res(digit1, |s: &str| f32::from_str(s)),
            )),
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

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration as StdDuration;
    #[test]
    fn parsing_units() {
        let d1: Duration = "P10Y10M10DT10H10M10S".parse().unwrap();
        assert_eq!(d1.to_std(), StdDuration::from_secs(342753010));
        let d2: Duration = "P10Y10M10DT10H10M10.5S".parse().unwrap();
        assert_eq!(d2.to_std(), StdDuration::from_millis(342753010500));
        let d3: Duration = "P10.5Y10M10DT10H10M10S".parse().unwrap();
        assert_eq!(d3.to_std(), StdDuration::from_secs(358531486));
        let d4: Duration = "P10Y10.5M10DT10H10M10S".parse().unwrap();
        assert_eq!(d4.to_std(), StdDuration::from_secs(344067154));
        let d5: Duration = "P10Y10M10.5DT10H10M10S".parse().unwrap();
        assert_eq!(d5.to_std(), StdDuration::from_secs(342796210));
        let d6: Duration = "P10Y10M10DT10.5H10M10S".parse().unwrap();
        assert_eq!(d6.to_std(), StdDuration::from_secs(342754810));
        let d7: Duration = "PT5.5H5.5M".parse().unwrap();
        assert_eq!(
            d7.to_std(),
            StdDuration::from_secs((5.5 * 60. * 60.) as u64 + (5.5 * 60.) as u64)
        );
    }
}
