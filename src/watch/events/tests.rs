use super::utc;
use proptest::prelude::*;

#[test]
fn utc_formats_epoch_current_date_and_leap_day() {
    assert_eq!(utc(0), "1970-01-01T00:00:00Z");
    assert_eq!(utc(1_793_102_400_000), "2026-10-27T12:00:00Z");
    assert_eq!(utc(951_782_400_000), "2000-02-29T00:00:00Z");
}

proptest! {
    #[test]
    fn utc_roundtrips_milliseconds(ms in 0i64..4_102_444_800_000) {
        let text = utc(ms);
        prop_assert_eq!(text.len(), 20);
        let fields: Vec<i64> = text.split(['-', 'T', ':', 'Z'])
            .filter(|s| !s.is_empty()).map(|s| s.parse().unwrap()).collect();
        let (year, month, day) = (fields[0], fields[1], fields[2]);
        let leap = |y| y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
        let mut days = (1970..year).map(|y| if leap(y) { 366 } else { 365 }).sum::<i64>();
        let lengths = [31, if leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        for m in 1..month { days += lengths[usize::try_from(m - 1).unwrap()]; }
        days += day - 1;
        prop_assert_eq!(days * 86_400 + fields[3] * 3_600 + fields[4] * 60 + fields[5], ms / 1_000);
        prop_assert_eq!((&text[4..5], &text[7..8], &text[10..11], &text[13..14], &text[16..17], &text[19..]), ("-", "-", "T", ":", ":", "Z"));
    }
}
