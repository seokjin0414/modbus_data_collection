use chrono::{DateTime, Duration, Timelike, Utc};
pub const MINUTE: &str = "MINUTE";
pub const HOUR: &str = "HOUR";
pub const DAY: &str = "DAY";

pub fn utc_now_minute() -> DateTime<Utc> {
    let now = Utc::now();

    now.with_second(0)
        .and_then(|dt| dt.with_nanosecond(0))
        .expect("Fail second, nanosecond Set 0.")
}

pub fn utc_now_hour() -> DateTime<Utc> {
    let now = Utc::now();

    now.with_minute(0)
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .expect("Fail second, nanosecond Set 0.")
}

pub fn utc_now_day() -> DateTime<Utc> {
    let now = Utc::now();

    now.with_hour(0)
        .and_then(|dt| dt.with_minute(0))
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .expect("Fail second, nanosecond Set 0.")
}

pub fn utc_now_ago(ago_seconds: i64, time_unit: &str) -> DateTime<Utc> {
    let date_time = match time_unit {
        DAY => utc_now_day(),
        HOUR => utc_now_hour(),
        _ => utc_now_minute(),
    };

    date_time - Duration::seconds(ago_seconds)
}