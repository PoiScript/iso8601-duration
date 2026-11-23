use std::convert::TryFrom;
use std::ops::{Add, Sub};

use chrono::{DateTime, Datelike, Duration as ChronoDuration, NaiveDate, TimeZone};

use crate::Duration;

/// Nanoseconds in one second, as `f64`.
const NANOS_PER_SECOND: f64 = 1_000_000_000.;

/// Converts seconds to a `chrono::Duration`, rounded to the nearest nanosecond.
///
/// Returns `None` if `seconds` is not finite or is out of the range of
/// `chrono::Duration`.
fn seconds_to_chrono_duration(seconds: f64) -> Option<ChronoDuration> {
    if !seconds.is_finite() {
        return None;
    }

    let whole = seconds.trunc();
    let nanos = ((seconds - whole) * NANOS_PER_SECOND).round();

    #[expect(
        clippy::cast_possible_truncation,
        reason = "`whole` is finite and integral; `as` saturates, and `try_seconds` rejects the saturated bounds"
    )]
    let whole = whole as i64;

    #[expect(
        clippy::cast_possible_truncation,
        reason = "`nanos` is integral and |nanos| <= 1e9, so it is exact in i64"
    )]
    let nanos = nanos as i64;

    ChronoDuration::try_seconds(whole)?.checked_add(&ChronoDuration::nanoseconds(nanos))
}

/// Returns the number of seconds between two dates, as `f64`.
fn seconds_between(from: NaiveDate, to: NaiveDate) -> f64 {
    let seconds = to.signed_duration_since(from).num_seconds();

    f64::from(i32::try_from(seconds).expect("a calendar month or year has fewer than 2^31 seconds"))
}

/// Returns the length of the calendar year that contains `date`, in seconds.
fn seconds_in_year(date: NaiveDate) -> f64 {
    let year = date.year();
    let first = NaiveDate::from_ymd_opt(year, 1, 1).expect("Date out of range");
    let next = NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("Date out of range");

    seconds_between(first, next)
}

/// Returns the length of the calendar month that contains `date`, in seconds.
fn seconds_in_month(date: NaiveDate) -> f64 {
    let (year, month) = (date.year(), date.month());
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("Date out of range");
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .expect("Date out of range");

    seconds_between(first, next)
}

/// Converts `duration` to a `chrono::Duration`, measured at `at`.
///
/// Years and months use the length of the calendar year and month that
/// contain `at`. Both `Add` and `Sub` use this rule.
///
/// # Panics
///
/// Panics if a calendar boundary is out of the range of `NaiveDate`, or if
/// the result is out of the range of `chrono::Duration`.
fn calendar_delta<Tz: TimeZone>(at: &DateTime<Tz>, duration: Duration) -> ChronoDuration {
    let date = at.date_naive();
    let mut seconds = duration.seconds_by_component().to_vec();

    if duration.year > 0.0 {
        seconds.push(duration.year * seconds_in_year(date));
    }

    if duration.month > 0.0 {
        seconds.push(duration.month * seconds_in_month(date));
    }

    sum_seconds(&seconds).expect("duration out of range of chrono::Duration")
}

/// Converts each value to a `chrono::Duration` and adds the results.
///
/// Each value is rounded on its own, so a long component does not take the
/// precision of a short one. Returns `None` on the first value or sum that
/// is out of range.
fn sum_seconds(seconds: &[f64]) -> Option<ChronoDuration> {
    seconds
        .iter()
        .try_fold(ChronoDuration::zero(), |total, &value| {
            total.checked_add(&seconds_to_chrono_duration(value)?)
        })
}

impl Duration {
    /// Convert `Duration` to `chrono::Duration`.
    ///
    /// This method returns `None` if the `Duration` contains `year` or
    /// `month`, if any component is not finite, or if the result is out of
    /// the range of `chrono::Duration`. Each component is rounded to the
    /// nearest nanosecond on its own, and the results are added. Negative
    /// components are allowed and give a negative contribution.
    pub fn to_chrono(&self) -> Option<ChronoDuration> {
        // we can't get the duration of year or month,
        // without knowing the start date.
        if self.year > 0.0 || self.month > 0.0 {
            return None;
        }

        sum_seconds(&self.seconds_by_component())
    }

    /// Convert `Duration` to `chrono::Duration` at given datetime.
    ///
    /// # Panics
    ///
    /// Panics if the result is out of the range of `DateTime` or
    /// `chrono::Duration`, as `DateTime + Duration` does.
    pub fn to_chrono_at_datetime<Tz: TimeZone>(&self, at: DateTime<Tz>) -> ChronoDuration {
        (at.clone() + *self) - at
    }
}

/// Adds a duration. Years and months use the calendar year and month of `self`.
///
/// # Panics
///
/// Panics if the result is out of the range of `DateTime`.
impl<Tz: TimeZone> Add<Duration> for DateTime<Tz> {
    type Output = DateTime<Tz>;

    fn add(self, rhs: Duration) -> Self {
        let delta = calendar_delta(&self, rhs);

        self + delta
    }
}

/// Subtracts a duration. Years and months use the calendar year and month of `self`.
///
/// # Panics
///
/// Panics if the result is out of the range of `DateTime`.
impl<Tz: TimeZone> Sub<Duration> for DateTime<Tz> {
    type Output = DateTime<Tz>;

    fn sub(self, rhs: Duration) -> Self {
        let delta = calendar_delta(&self, rhs);

        self - delta
    }
}

#[test]
fn test_chrono() {
    use chrono::Utc;

    fn ymd(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(
            NaiveDate::from_ymd_opt(y, m, d)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            Utc,
        )
    }

    let quarter: Duration = "P0.25Y".parse().unwrap();

    assert_eq!(
        (ymd(2000, 6, 1) + quarter).to_rfc3339(),
        "2000-08-31T12:00:00+00:00" // 91.5days
    );
    assert_eq!(
        (ymd(2001, 6, 1) + quarter).to_rfc3339(),
        "2001-08-31T06:00:00+00:00" // 91.25days
    );

    let half_month: Duration = "P0.5M".parse().unwrap();

    assert_eq!(
        (ymd(2001, 2, 1) + half_month).to_rfc3339(),
        "2001-02-15T00:00:00+00:00" // 14days
    );
    assert_eq!(
        (ymd(2001, 4, 1) + half_month).to_rfc3339(),
        "2001-04-16T00:00:00+00:00" // 15days
    );

    assert_eq!(
        (ymd(2001, 4, 16) - half_month).to_rfc3339(),
        "2001-04-01T00:00:00+00:00" // 15days
    );
    assert_eq!(
        (ymd(2000, 8, 31) - quarter).to_rfc3339(),
        "2000-05-31T12:00:00+00:00" // 91.5days
    );
    assert_eq!(
        (ymd(2001, 3, 1) - "P1DT1H".parse::<Duration>().unwrap()).to_rfc3339(),
        "2001-02-27T23:00:00+00:00"
    );

    let week: Duration = "P1W".parse().unwrap();
    assert_eq!(week.to_chrono(), Some(ChronoDuration::weeks(1)));
}

/// Kani proof harnesses for the `chrono` conversions.
#[cfg(kani)]
mod proofs {
    use super::seconds_to_chrono_duration;

    /// The seconds conversion does not panic for any `f64`.
    ///
    /// `to_chrono` and `sum_seconds` only add its results with `checked_add`.
    #[kani::proof]
    fn seconds_to_chrono_duration_never_panics() {
        let _ = seconds_to_chrono_duration(kani::any());
    }
}
