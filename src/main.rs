use std::thread;
use std::collections::HashMap;
use tokio::sync::watch;
use tokio::runtime::Runtime;
use std::ops::Add;
use tokio::time::{sleep_until, Instant, Duration};
use chrono::{NaiveTime, Timelike, DateTime, Local, Datelike};
use num_traits::{FromPrimitive};
use tokio::sync::mpsc::{Receiver, Sender};
use std::thread::sleep;

struct Operation {
    time: WeekTime,
    operation: &'static (dyn Fn() + Sync + Send),
}

struct WeekTime {
    day: WeekDay,
    time: NaiveTime
}

impl WeekTime {
    fn interval(&self, other: &Self) -> Duration {
        Duration::from_secs(
            ((other.to_seconds() as i32 - self.to_seconds() as i32) % (24*3600*7)) as u64)
    }

    fn interval_from_now(&self) -> Duration {
        Self::now().interval(self)
    }

    fn now() -> WeekTime {
        let now = Local::now();
        WeekTime{
            day: FromPrimitive::from_u32(now.weekday().num_days_from_monday()).unwrap(),
            time: now.time()
        }
    }

    fn to_seconds(&self) -> u32 {
        let seconds_of_day = 24 * 3600;
        (self.day as u32) * seconds_of_day + self.time.num_seconds_from_midnight()
    }

    fn from_seconds(seconds: u32) -> Self {
        let days = seconds / (24*3600);
        let seconds = seconds - (days * (24*3600));
        WeekTime{
            day: FromPrimitive::from_u32(days).unwrap(),
            time: NaiveTime::from_num_seconds_from_midnight(seconds, 0)}
    }
}

#[derive(Copy, Clone, num_derive::FromPrimitive)]
enum WeekDay {
    Monday=0,
    Tuesday=1,
    Wednesday=2,
    Thursday=3,
    Friday=4,
    Saturday=5,
    Sunday=6
}

struct Scheduler {
    sender: Sender<Operation>
}

impl Scheduler {
    fn new() -> Scheduler {
        let (tx, rx) = tokio::sync::mpsc::channel::<Operation>(1000000);
        tokio::spawn(Self::start_scheduler(tx.clone(), rx));

        Scheduler {
            sender: tx
        }
    }

    async fn start_scheduler(tx: Sender<Operation>, mut rx: Receiver<Operation>, ) -> Option<()> {
        loop {
            let operation = rx.recv().await?;

            let tx = tx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(operation.time.interval_from_now()).await;
                (operation.operation)();
                tokio::time::sleep(Duration::from_secs(2)).await;
                tx.send(operation).await;
            });
        }
        Some(())
    }

    async fn initial_scheduling(&self, operations: Vec<Operation>) {
        for operation in operations {
            self.sender.send(operation).await;
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let mut scheduler = Scheduler::new();
    scheduler.initial_scheduling(vec![
        Operation {
            time: WeekTime::from_seconds(WeekTime::now().to_seconds() + 3),
            operation: &|| println!("3 seconds passed"),
        },
        Operation {
            time: WeekTime::from_seconds(WeekTime::now().to_seconds() + 4),
            operation: &|| println!("4 seconds passed"),
        },
        Operation {
            time: WeekTime::from_seconds(WeekTime::now().to_seconds() + 5),
            operation: &|| println!("5 seconds passed"),
        },
        Operation {
            time: WeekTime::from_seconds(WeekTime::now().to_seconds() + 6),
            operation: &|| println!("6 seconds passed"),
        },
        Operation {
            time: WeekTime::from_seconds(WeekTime::now().to_seconds() + 7),
            operation: &|| println!("7 seconds passed"),
        }
    ]).await;

    thread::sleep(Duration::from_secs(3));

    thread::sleep(Duration::from_secs(5));
}
