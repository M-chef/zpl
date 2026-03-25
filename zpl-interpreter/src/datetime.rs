use std::time::{SystemTime, UNIX_EPOCH};

use crate::SetRealTimeClock;

pub fn format_timestamp(
    fmt: &str,
    escape_chars: &[char],
    real_time_clock_setup: &SetRealTimeClock,
) -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (year, month, day, hour, min, sec) = decompose(secs);

    let (year, month, day, hour, min, sec) =
        replace_with_setup(year, month, day, hour, min, sec, real_time_clock_setup);

    let mut out = String::with_capacity(fmt.len());
    let mut chars = fmt.chars().peekable();

    while let Some(c) = chars.next() {
        if !escape_chars.contains(&c) {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('Y') => out.push_str(&format!("{:04}", year)),
            Some('m') => out.push_str(&format!("{:02}", month)),
            Some('d') => out.push_str(&format!("{:02}", day)),
            Some('H') => out.push_str(&format!("{:02}", hour)),
            Some('M') => out.push_str(&format!("{:02}", min)),
            Some('S') => out.push_str(&format!("{:02}", sec)),
            Some(other) => {
                out.push(c);
                out.push(other);
            } // unknown token, pass through
            None => out.push(c), // trailing % at end of string
        }
    }

    out
}

fn replace_with_setup(
    year: u64,
    month: u64,
    day: u64,
    hour: u64,
    min: u64,
    sec: u64,
    real_time_clock_setup: &SetRealTimeClock,
) -> (usize, u8, u8, u8, u8, u8) {
    (
        real_time_clock_setup.year.unwrap_or(year as usize),
        real_time_clock_setup.month.unwrap_or(month as u8),
        real_time_clock_setup.day.unwrap_or(day as u8),
        real_time_clock_setup.hour.unwrap_or(hour as u8),
        real_time_clock_setup.minute.unwrap_or(min as u8),
        real_time_clock_setup.second.unwrap_or(sec as u8),
    )
}

fn decompose(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let sec = secs % 60;
    let min = (secs / 60) % 60;
    let hour = (secs / 3600) % 24;

    let mut days = secs / 86400;
    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let month_lengths = [
        31,
        if is_leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u64;
    for len in month_lengths {
        if days < len {
            break;
        }
        days -= len;
        month += 1;
    }

    (year, month, days + 1, hour, min, sec)
}

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use crate::{SetRealTimeClock, datetime::format_timestamp};

    #[test]
    fn should_format_timestamp_for_one_escape_char() {
        let real_time_clock_setup = SetRealTimeClock::default();
        let res = format_timestamp("%Y.%d.%m %H:%M:%S", &['%'], &real_time_clock_setup);
        println!("{res}")
    }

    #[test]
    fn should_format_timestamp_for_other_escape_char() {
        let real_time_clock_setup = SetRealTimeClock::default();
        let res = format_timestamp("+Y+.+d.+m +H:+M:+S", &['+'], &real_time_clock_setup);
        println!("{res}")
    }

    #[test]
    fn should_format_timestamp_for_set_real_time_clock() {
        let real_time_clock_setup = SetRealTimeClock {
            month: Some(1),
            day: Some(1),
            year: Some(1),
            hour: Some(1),
            minute: Some(1),
            second: Some(1),
            ..Default::default()
        };
        let res = format_timestamp("+Y+.+d.+m +H:+M:+S", &['+'], &real_time_clock_setup);
        assert_eq!(&res, "0001+.01.01 01:01:01")
    }
}
