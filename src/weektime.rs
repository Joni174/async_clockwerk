use chrono::{NaiveTime, Local, Timelike, Datelike};
use tokio::time::Duration;
use num_traits::FromPrimitive;
use num_derive::FromPrimitive;

#[derive(Debug)]
pub struct WeekTime {
    day: Weekday,
    time: NaiveTime,
}

impl WeekTime {
    pub fn new(day: Weekday, time: NaiveTime) -> WeekTime {
        WeekTime { day, time }
    }

    /// get get interval between self and other
    /// When times are output whole week
    fn interval(&self, other: &Self) -> Duration {
        let other = other.to_seconds() as i32;
        let this = self.to_seconds() as i32;
        let seconds_per_week = 24 * 3600 * 7 as i32;
        if this == other {
            Duration::from_secs(seconds_per_week as u64)
        } else {
            Duration::from_secs(
                ((((other - this) % seconds_per_week) + seconds_per_week) % seconds_per_week) as u64)
        }
    }

    pub(crate) fn interval_from_now(&self) -> Duration {
        Self::now().interval(self)
    }

    pub(crate) fn from_seconds(seconds: u32) -> WeekTime {
        let seconds = seconds % (3600 * 24 * 7);
        let days = seconds / (3600 * 24);
        let seconds = seconds - (days * 3600 * 24);
        WeekTime {
            day: FromPrimitive::from_u32(days).unwrap(),
            time: NaiveTime::from_num_seconds_from_midnight(seconds, 0),
        }
    }

    pub(crate) fn now() -> WeekTime {
        let now = Local::now();
        WeekTime {
            day: FromPrimitive::from_u32(now.weekday().num_days_from_monday()).unwrap(),
            time: now.time(),
        }
    }

    pub(crate) fn to_seconds(&self) -> u32 {
        let seconds_of_day = 24 * 3600;
        (self.day as u32) * seconds_of_day + self.time.num_seconds_from_midnight()
    }
}

#[derive(Copy, Clone, FromPrimitive, Debug)]
pub enum Weekday {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}

#[cfg(test)]
mod test {
    use crate::weektime::WeekTime;
    use crate::weektime::Weekday::Monday;
    use chrono::NaiveTime;
    use tokio::time::Duration;

    #[test]
    fn test_interval() {
        let t1 = WeekTime::new(Monday, NaiveTime::from_hms(10, 0, 0));
        let t2 = WeekTime::new(Monday, NaiveTime::from_hms(10, 20, 0));

        let interval = t1.interval(&t2);
        println!("{:?}", interval);
        assert_eq!(interval, Duration::new(20 * 60, 0));
    }

    #[test]
    fn test_interval_carry() {
        let t1 = WeekTime::new(Monday, NaiveTime::from_hms(10, 0, 0));
        let t2 = WeekTime::new(Monday, NaiveTime::from_hms(10, 20, 0));

        let interval = t2.interval(&t1);
        println!("{:?}", interval);
        assert_eq!(interval, Duration::from_secs(3600 * 24 * 6 + 3600 * 23 + 60 * 40));
    }
}
