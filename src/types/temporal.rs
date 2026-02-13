//! Temporal types: Date, LocalTime, ZonedTime, LocalDateTime, ZonedDateTime, Duration.

use crate::proto;

/// Calendar date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    /// Year (can be negative for BCE).
    pub year: i32,
    /// Month (1-12).
    pub month: u32,
    /// Day (1-31).
    pub day: u32,
}

/// Time without timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    /// Hour (0-23).
    pub hour: u32,
    /// Minute (0-59).
    pub minute: u32,
    /// Second (0-59).
    pub second: u32,
    /// Nanosecond (0-999999999).
    pub nanosecond: u32,
}

/// Time with UTC offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZonedTime {
    /// The time component.
    pub time: LocalTime,
    /// UTC offset in minutes.
    pub offset_minutes: i32,
}

/// Date and time without timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalDateTime {
    /// The date component.
    pub date: Date,
    /// The time component.
    pub time: LocalTime,
}

/// Date and time with UTC offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZonedDateTime {
    /// The date component.
    pub date: Date,
    /// The time component.
    pub time: LocalTime,
    /// UTC offset in minutes.
    pub offset_minutes: i32,
}

/// Temporal duration with two components per ISO/IEC 39075.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    /// Year-to-month component.
    pub months: i64,
    /// Day-to-second component in nanoseconds.
    pub nanoseconds: i64,
}

// ============================================================================
// Proto conversions
// ============================================================================

impl From<proto::Date> for Date {
    fn from(p: proto::Date) -> Self {
        Self {
            year: p.year,
            month: p.month,
            day: p.day,
        }
    }
}

impl From<Date> for proto::Date {
    fn from(d: Date) -> Self {
        Self {
            year: d.year,
            month: d.month,
            day: d.day,
        }
    }
}

impl From<proto::LocalTime> for LocalTime {
    fn from(p: proto::LocalTime) -> Self {
        Self {
            hour: p.hour,
            minute: p.minute,
            second: p.second,
            nanosecond: p.nanosecond,
        }
    }
}

impl From<LocalTime> for proto::LocalTime {
    fn from(t: LocalTime) -> Self {
        Self {
            hour: t.hour,
            minute: t.minute,
            second: t.second,
            nanosecond: t.nanosecond,
        }
    }
}

impl From<proto::ZonedTime> for ZonedTime {
    fn from(p: proto::ZonedTime) -> Self {
        Self {
            time: p.time.map_or(
                LocalTime {
                    hour: 0,
                    minute: 0,
                    second: 0,
                    nanosecond: 0,
                },
                LocalTime::from,
            ),
            offset_minutes: p.offset_minutes,
        }
    }
}

impl From<ZonedTime> for proto::ZonedTime {
    fn from(t: ZonedTime) -> Self {
        Self {
            time: Some(t.time.into()),
            offset_minutes: t.offset_minutes,
        }
    }
}

impl From<proto::LocalDateTime> for LocalDateTime {
    fn from(p: proto::LocalDateTime) -> Self {
        Self {
            date: p.date.map_or(
                Date {
                    year: 0,
                    month: 0,
                    day: 0,
                },
                Date::from,
            ),
            time: p.time.map_or(
                LocalTime {
                    hour: 0,
                    minute: 0,
                    second: 0,
                    nanosecond: 0,
                },
                LocalTime::from,
            ),
        }
    }
}

impl From<LocalDateTime> for proto::LocalDateTime {
    fn from(dt: LocalDateTime) -> Self {
        Self {
            date: Some(dt.date.into()),
            time: Some(dt.time.into()),
        }
    }
}

impl From<proto::ZonedDateTime> for ZonedDateTime {
    fn from(p: proto::ZonedDateTime) -> Self {
        Self {
            date: p.date.map_or(
                Date {
                    year: 0,
                    month: 0,
                    day: 0,
                },
                Date::from,
            ),
            time: p.time.map_or(
                LocalTime {
                    hour: 0,
                    minute: 0,
                    second: 0,
                    nanosecond: 0,
                },
                LocalTime::from,
            ),
            offset_minutes: p.offset_minutes,
        }
    }
}

impl From<ZonedDateTime> for proto::ZonedDateTime {
    fn from(dt: ZonedDateTime) -> Self {
        Self {
            date: Some(dt.date.into()),
            time: Some(dt.time.into()),
            offset_minutes: dt.offset_minutes,
        }
    }
}

impl From<proto::Duration> for Duration {
    fn from(p: proto::Duration) -> Self {
        Self {
            months: p.months,
            nanoseconds: p.nanoseconds,
        }
    }
}

impl From<Duration> for proto::Duration {
    fn from(d: Duration) -> Self {
        Self {
            months: d.months,
            nanoseconds: d.nanoseconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip_date(d: Date) {
        let p: proto::Date = d.into();
        let back: Date = p.into();
        assert_eq!(d, back);
    }

    fn round_trip_local_time(t: LocalTime) {
        let p: proto::LocalTime = t.into();
        let back: LocalTime = p.into();
        assert_eq!(t, back);
    }

    #[test]
    fn date_round_trip() {
        round_trip_date(Date {
            year: 2026,
            month: 2,
            day: 13,
        });
        round_trip_date(Date {
            year: -500,
            month: 1,
            day: 1,
        });
    }

    #[test]
    fn local_time_round_trip() {
        round_trip_local_time(LocalTime {
            hour: 14,
            minute: 30,
            second: 45,
            nanosecond: 123_456_789,
        });
    }

    #[test]
    fn zoned_time_round_trip() {
        let t = ZonedTime {
            time: LocalTime {
                hour: 10,
                minute: 0,
                second: 0,
                nanosecond: 0,
            },
            offset_minutes: 60,
        };
        let p: proto::ZonedTime = t.into();
        let back: ZonedTime = p.into();
        assert_eq!(t, back);
    }

    #[test]
    fn local_datetime_round_trip() {
        let dt = LocalDateTime {
            date: Date {
                year: 2026,
                month: 2,
                day: 13,
            },
            time: LocalTime {
                hour: 14,
                minute: 30,
                second: 0,
                nanosecond: 0,
            },
        };
        let p: proto::LocalDateTime = dt.into();
        let back: LocalDateTime = p.into();
        assert_eq!(dt, back);
    }

    #[test]
    fn zoned_datetime_round_trip() {
        let dt = ZonedDateTime {
            date: Date {
                year: 2026,
                month: 2,
                day: 13,
            },
            time: LocalTime {
                hour: 14,
                minute: 30,
                second: 0,
                nanosecond: 0,
            },
            offset_minutes: -300,
        };
        let p: proto::ZonedDateTime = dt.into();
        let back: ZonedDateTime = p.into();
        assert_eq!(dt, back);
    }

    #[test]
    fn duration_round_trip() {
        let d = Duration {
            months: 14,
            nanoseconds: 86_400_000_000_000,
        };
        let p: proto::Duration = d.into();
        let back: Duration = p.into();
        assert_eq!(d, back);
    }
}
