use std::thread;
use std::collections::HashMap;
use tokio::sync::watch;
use tokio::runtime::Runtime;
use std::ops::Add;
use tokio::time::{sleep_until, Instant, Duration};
use chrono::{NaiveTime, Timelike};

struct Operation {
    id: u32,
    time: WeekTime,
    operation: &'static (dyn Fn() + Sync + Send),
}

struct WeekTime {
    day: WeekDay,
    time: NaiveTime
}

impl WeekTime {
    fn interval(&self, other: &Self) -> Duration {
        Duration::from_secs((other.to_seconds() - self.to_seconds()) as u64)
    }

    fn to_seconds(&self) -> u32 {
        let seconds_of_day = 24 * 3600;
        (self.day as u32) * seconds_of_day + self.time.num_seconds_from_midnight()
    }
}

#[derive(Copy, Clone)]
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
    operations: Vec<Operation>,
    scheduled_operations: HashMap<u32, watch::Sender<()>>, // shutdown handles
    runtime: Runtime,
}

impl Scheduler {
    fn new(operations: Vec<Operation>) -> Scheduler {
        Scheduler {
            operations,
            scheduled_operations: HashMap::new(),
            runtime: Runtime::new().unwrap(),
        }
    }

    fn initial_scheduling(&mut self) {
        for operation in &self.operations {
            let op = operation.operation;
            let time = operation.time.clone();
            let (shutdown_handle, mut shutdown_receiver) = watch::channel(());
            self.scheduled_operations.insert(operation.id, shutdown_handle);

            self.runtime.spawn(
                async move {
                    tokio::select! {
                        _ = shutdown_receiver.changed() => {}
                        _ = sleep_until(time) => {(op)();}
                    }
                }
            );
        }
    }
}

fn main() {
    println!("Hello, world!");

    let mut scheduler = Scheduler::new(vec![
        Operation {
            id: 0,
            time: Instant::now().add(Duration::from_secs(3)),
            operation: &|| println!("3 seconds passed"),
        },
        Operation {
            id: 1,
            time: Instant::now().add(Duration::from_secs(4)),
            operation: &|| println!("4 seconds passed"),
        },
        Operation {
            id: 2,
            time: Instant::now().add(Duration::from_secs(5)),
            operation: &|| println!("5 seconds passed"),
        },
        Operation {
            id: 3,
            time: Instant::now().add(Duration::from_secs(6)),
            operation: &|| println!("6 seconds passed"),
        },
        Operation {
            id: 4,
            time: Instant::now().add(Duration::from_secs(7)),
            operation: &|| println!("7 seconds passed"),
        }
    ]);

    scheduler.initial_scheduling();

    thread::sleep(Duration::from_secs(3));

    thread::sleep(Duration::from_secs(5));
}
