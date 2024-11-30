use jiff::{tz::TimeZone, Timestamp, Zoned};

// Because the pi might be set to UTC, we need to manually add the tz
pub(crate) fn now() -> Zoned {
    let local = TimeZone::get("Amsterdam").unwrap();
    Timestamp::now().to_zoned(local)
}
