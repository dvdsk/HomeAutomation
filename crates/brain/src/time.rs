use jiff::{tz::TimeZone, Timestamp, Zoned};

// Because the pi might be set to UTC, we need to manually add the tz
pub(crate) fn now() -> Zoned {
    let local = TimeZone::get("Europe/Amsterdam").unwrap();
    Timestamp::now().to_zoned(local)
}

pub(crate) fn to_datetime(hour: i8, min: i8) -> Zoned {
    let now = crate::time::now();
    let job_time = now.with().hour(hour).minute(min).second(0).build().unwrap();

    if job_time < now {
        let tomorrow = now.tomorrow().unwrap();
        tomorrow
            .with()
            .hour(hour)
            .minute(min)
            .second(0)
            .build()
            .unwrap()
    } else {
        job_time
    }
}
