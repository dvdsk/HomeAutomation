use std::time::Duration;

use super::duration;

pub(crate) enum FmtScale {
    RelativeDuration,
    HourMinute,
    DayHour,
    MonthDayHour,
    YearMonthDayHour,
}

impl FmtScale {
    pub fn optimal_for(t: jiff::Timestamp, elapsed: Duration) -> Self {
        pub(crate) const HOUR: Duration = Duration::from_secs(60 * 60);

        let t = t.to_zoned(jiff::tz::TimeZone::system());
        let one_month_ago = t.checked_sub(jiff::Span::new().months(1)).unwrap();
        let one_year_ago = t.checked_sub(jiff::Span::new().years(1)).unwrap();

        if elapsed < 2 * HOUR {
            Self::RelativeDuration
        } else if elapsed < 24 * HOUR {
            Self::HourMinute
        } else if t < one_month_ago {
            Self::DayHour
        } else if t < one_year_ago {
            Self::MonthDayHour
        } else {
            Self::YearMonthDayHour
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

        match self {
            FmtScale::RelativeDuration => duration(elapsed.as_secs_f64()) + duration_postfix,
            FmtScale::HourMinute => t.strftime("%H:%M").to_string(),
            FmtScale::DayHour => t.strftime("%D %H").to_string(),
            FmtScale::MonthDayHour => t.strftime("%M %D %H").to_string(),
            FmtScale::YearMonthDayHour => t.strftime("%Y %M %D %H").to_string(),
        }
    }
}
