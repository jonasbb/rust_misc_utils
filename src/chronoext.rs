use chrono::{DateTime, Duration, TimeZone};

pub trait RoundTime: Sized {
    fn round_to_seconds(self, seconds: u64) -> Option<Self>;
    fn round_to_millis(self, millis: u32) -> Option<Self>;
    fn round_to_micros(self, micros: u32) -> Option<Self>;
    fn round_to_nanos(self, nanos: u32) -> Option<Self>;
}

impl<Tz: TimeZone> RoundTime for DateTime<Tz> {
    fn round_to_seconds(self, seconds: u64) -> Option<Self> {
        let mut duration = Duration::nanoseconds(self.timestamp_subsec_nanos() as i64);
        if seconds <= i64::max_value() as u64 {
            // seconds can be converted to i64 and might round the value to something
            duration = duration + Duration::seconds(self.timestamp() % (seconds as i64));
        }
        self.checked_sub_signed(duration)
    }

    fn round_to_millis(self, millis: u32) -> Option<Self> {
        let millis_per_second = 1_000;
        let millis_per_nano = 1_000_000;
        if millis >= millis_per_second {
            return None;
        }
        let duration = Duration::nanoseconds(
            self.timestamp_subsec_nanos() as i64 % (millis_per_nano * millis as i64),
        );
        self.checked_sub_signed(duration)
    }

    fn round_to_micros(self, micros: u32) -> Option<Self> {
        unimplemented!()
    }

    fn round_to_nanos(self, nanos: u32) -> Option<Self> {
        unimplemented!()
    }
}
// fn round_datetime<T: ::chrono::TimeZone>(datetime: DateTime<T>, seconds: u32) -> DateTime<T> {
//     let nanoseconds_per_second = 1_000_000_000;
//     let duration = ::chrono::Duration::nanoseconds(
//         datetime.timestamp_subsec_nanos() as i64 +
//             (datetime.timestamp() % seconds as i64) * nanoseconds_per_second,
//     );
//     datetime.checked_sub_signed(duration).expect(
//         "The duration time must be smaller than the datetime, so no overflow.",
//     )
// }
