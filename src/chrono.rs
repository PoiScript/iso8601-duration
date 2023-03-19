use std::ops::Add;

use chrono::{DateTime, Datelike, Duration as ChronoDuration, NaiveDate, TimeZone};

use crate::Duration;

fn seconds_to_chrono_duration(seconds: f32) -> ChronoDuration {
    let nanoseconds = seconds.fract() * 1_000_000_000.;
    let seconds = seconds.trunc();

    ChronoDuration::seconds(seconds as i64) + ChronoDuration::nanoseconds(nanoseconds as i64)
}

impl Duration {
    pub fn to_chrono(&self) -> Option<ChronoDuration> {
        // we can't get the duration of year or month,
        // without knowing the start date.
        if self.year > 0.0 || self.month > 0.0 {
            return None;
        }

        let seconds =
            self.day * 60. * 60. * 24. + self.hour * 60. * 60. + self.minute * 60. + self.second;

        Some(seconds_to_chrono_duration(seconds))
    }
}

impl<Tz: TimeZone> Add<Duration> for DateTime<Tz> {
    type Output = DateTime<Tz>;

    fn add(self, rhs: Duration) -> Self {
        let mut d = ChronoDuration::zero();

        if rhs.year > 0.0 {
            let year = self.date_naive().year();

            let seconds_in_this_year = NaiveDate::from_ymd_opt(year + 1, 1, 1)
                .expect("Date out of range")
                .signed_duration_since(
                    NaiveDate::from_ymd_opt(year, 1, 1).expect("Date out of range"),
                )
                .num_seconds();

            d = d + seconds_to_chrono_duration(rhs.year * seconds_in_this_year as f32)
        }

        if rhs.month > 0.0 {
            let year = self.date_naive().year();
            let month = self.date_naive().month();

            let seconds_in_this_month = NaiveDate::from_ymd_opt(
                if month == 12 { year + 1 } else { year },
                if month == 12 { 1 } else { month + 1 },
                1,
            )
            .expect("Date out of range")
            .signed_duration_since(
                NaiveDate::from_ymd_opt(year, month, 1).expect("Date out of range"),
            )
            .num_seconds();

            d = d + seconds_to_chrono_duration(rhs.month * seconds_in_this_month as f32)
        }

        d = d + seconds_to_chrono_duration(
            rhs.day * 60. * 60. * 24. + rhs.hour * 60. * 60. + rhs.minute * 60. + rhs.second,
        );

        self + d
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

    let week: Duration = "P1W".parse().unwrap();
    assert_eq!(week.to_chrono(), Some(ChronoDuration::weeks(1)));
}
