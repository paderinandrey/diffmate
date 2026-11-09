pub(crate) fn utc(epoch_ms: i64) -> String {
    let seconds = epoch_ms.div_euclid(1_000);
    let days = seconds.div_euclid(86_400) + 719_468;
    let era = days.div_euclid(146_097);
    let day = days - era * 146_097;
    let year = (day - day / 1_460 + day / 36_524 - day / 146_096) / 365;
    let ordinal = day - (365 * year + year / 4 - year / 100);
    let month = (5 * ordinal + 2) / 153;
    let date = ordinal - (153 * month + 2) / 5 + 1;
    let month = month + if month < 10 { 3 } else { -9 };
    let year = year + era * 400 + i64::from(month <= 2);
    let time = seconds.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{date:02}T{:02}:{:02}:{:02}Z",
        time / 3_600,
        time / 60 % 60,
        time % 60
    )
}
