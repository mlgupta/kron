/// Round elapsed minutes UP to the next 15-minute increment.
///
/// `ceil(actual_minutes / 15) * 15`
pub fn round_up_minutes(actual_minutes: u64) -> u64 {
    if actual_minutes == 0 {
        0
    } else {
        actual_minutes.div_ceil(15) * 15
    }
}

/// Round an elapsed duration using fractional minutes derived from seconds.
pub fn round_up_from_seconds(actual_seconds: i64) -> i64 {
    let seconds = actual_seconds.max(0) as u64;
    if seconds == 0 {
        return 0;
    }
    let actual_minutes = seconds as f64 / 60.0;
    (actual_minutes / 15.0).ceil() as i64 * 15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_up_to_next_fifteen_minutes() {
        let cases = [
            (1, 15),
            (7, 15),
            (14, 15),
            (15, 15),
            (16, 30),
            (22, 30),
            (29, 30),
            (30, 30),
            (31, 45),
            (44, 45),
            (45, 45),
            (46, 60),
            (60, 60),
            (61, 75),
        ];

        for (actual, expected) in cases {
            assert_eq!(
                round_up_minutes(actual),
                expected,
                "{actual} minutes should round to {expected}"
            );
        }
    }

    #[test]
    fn zero_minutes_stays_zero() {
        assert_eq!(round_up_minutes(0), 0);
        assert_eq!(round_up_from_seconds(0), 0);
    }

    #[test]
    fn sub_minute_durations_round_to_fifteen() {
        assert_eq!(round_up_from_seconds(1), 15);
        assert_eq!(round_up_from_seconds(59), 15);
        assert_eq!(round_up_from_seconds(60), 15);
    }

    #[test]
    fn exact_quarter_hours_from_seconds() {
        assert_eq!(round_up_from_seconds(15 * 60), 15);
        assert_eq!(round_up_from_seconds(15 * 60 + 1), 30);
        assert_eq!(round_up_from_seconds(61 * 60), 75);
    }
}
