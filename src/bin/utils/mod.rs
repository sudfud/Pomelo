pub (crate) fn secs_to_timestamp(seconds: u64, include_hour: bool) -> String {
    let minutes = seconds / 60;
    let hours = minutes / 60;

    let mut result = String::new();

    if include_hour {
        result.push_str(&format!("{:02}:", hours));
    }

    result.push_str(&format!("{:02}:{:02}", minutes % 60, seconds % 60));

    result
}

mod tests {

    #[test]
    fn test_timestamp() {
        use super::secs_to_timestamp;

        assert_eq!(secs_to_timestamp(0, false), "00:00");
        assert_eq!(secs_to_timestamp(0, true), "00:00:00");

        assert_eq!(secs_to_timestamp(5, false), "00:05");
        assert_eq!(secs_to_timestamp(5, true), "00:00:05");

        assert_eq!(secs_to_timestamp(90, false), "01:30");
        assert_eq!(secs_to_timestamp(90, true), "00:01:30");

        assert_eq!(secs_to_timestamp(7330, false), "02:10");
        assert_eq!(secs_to_timestamp(7330, true), "02:02:10");

        assert_eq!(secs_to_timestamp(360_000, true), "100:00:00");
    }
}