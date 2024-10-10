use std::fmt::Display;
use std::time::Duration;

use super::duration;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FmtScale {
    RelativeDuration,
    HourMinute,
    DayHour,
    MonthDay,
    YearMonth,
}

impl Display for FmtScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FmtScale::RelativeDuration => f.write_str("elapsed"),
            FmtScale::HourMinute => f.write_str("<hour>:<minute>"),
            FmtScale::DayHour => f.write_str("<day (date)> <hour>"),
            FmtScale::MonthDay => f.write_str("<month>/<day>"),
            FmtScale::YearMonth => f.write_str("<year>/<month>"),
        }
    }
}

impl FmtScale {
    /// figure out optimal time description for every label point
    /// for example use 15:30 for something 3h ago but 2024/05 for
    /// a label 3 years ago.
    pub fn optimal_for(t: jiff::Timestamp, elapsed: Duration) -> Self {
        pub(crate) const HOUR: Duration = Duration::from_secs(60 * 60);

        let t = t.to_zoned(jiff::tz::TimeZone::system());
        let one_month_ago = jiff::Zoned::now()
            .checked_sub(jiff::Span::new().months(1))
            .unwrap();
        let one_year_ago = jiff::Zoned::now()
            .checked_sub(jiff::Span::new().years(1))
            .unwrap();

        if elapsed < 2 * HOUR {
            Self::RelativeDuration
        } else if elapsed < 24 * HOUR {
            Self::HourMinute
        } else if t > one_month_ago {
            Self::DayHour
        } else if t > one_year_ago {
            Self::MonthDay
        } else {
            Self::YearMonth
        }
    }

    pub(crate) fn render(
        &self,
        t: jiff::Timestamp,
        elapsed: Duration,
        duration_postfix: &str,
    ) -> String {
        if elapsed.is_zero() {
            return "now".to_string();
        }

        let t = t.to_zoned(jiff::tz::TimeZone::system());
        match self {
            FmtScale::RelativeDuration => duration(elapsed.as_secs_f64()) + duration_postfix,
            FmtScale::HourMinute => t.strftime("%H:%M").to_string(),
            FmtScale::DayHour => t.strftime("%dth %Hh").to_string(),
            FmtScale::MonthDay => t.strftime("%M/%d").to_string(),
            FmtScale::YearMonth => t.strftime("%Y/%m").to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn month_ago() {
        let t = jiff::Timestamp::now() - jiff::Span::new().hours(40 * 24);
        let elapsed = jiff::Timestamp::now().duration_since(t).unsigned_abs();
        let res = FmtScale::optimal_for(t, elapsed);
        assert_eq!(res, FmtScale::MonthDay)
    }
}
