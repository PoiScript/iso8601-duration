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
    Err, Finish, IResult,
};

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

    pub fn to_std(&self) -> StdDuration {
        StdDuration::from_secs_f32(
            self.year * 60. * 60. * 24. * 30. * 12.
                + self.month * 60. * 60. * 24. * 30.
                + self.day * 60. * 60. * 24.
                + self.hour * 60. * 60.
                + self.minute * 60.
                + self.second,
        )
    }

    pub fn parse(input: &str) -> Result<Duration, Error<&str>> {
        all_consuming(preceded(
            tag("P"),
            alt((parse_week_format, parse_basic_format)),
        ))(input)
        .finish()
        .map(|(_, duration)| duration)
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
