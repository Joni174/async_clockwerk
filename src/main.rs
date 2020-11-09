use std::thread;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use tokio::sync::{watch};
use tokio::runtime::Runtime;
use tokio::time::{Duration};
use chrono::{NaiveTime, Timelike, Local};
use num_traits::{FromPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use std::iter::FromIterator;

type Closure = &'static (dyn Fn() + Sync + Send);
type Schedule = Arc<Mutex<HashMap<usize, watch::Sender<()>>>>;

struct Operation {
    id: usize,
    operation: Closure,
    weektime: WeekTime
}

impl Clone for Operation {
    fn clone(&self) -> Self {
        Operation{id: self.id, operation: self.operation, weektime: self.weektime.clone()}
    }

    fn clone_from(&mut self, source: &Self) {
        unimplemented!()
    }
}

#[derive (Clone)]
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

    fn now() -> Self {
        let time = Local::now();
        WeekTime {
            day: FromPrimitive::from_u32(3).unwrap(),
            time: time.time()
        }
    }

    fn add_duration(&self, delta: &Duration) -> WeekTime {
        let seconds = (self.to_seconds() + delta.as_secs() as u32) % (3600*24*7);
        let days = seconds / (3600 * 24);
        let seconds = seconds - (days * 3600 * 24);
        WeekTime {
            day: FromPrimitive::from_u32(days).unwrap(),
            time: NaiveTime::from_num_seconds_from_midnight(seconds,0)
        }
    }
}

#[derive(Copy, Clone, FromPrimitive, ToPrimitive)]
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
    schedule: Schedule, // shutdown handles
    runtime: Arc<Runtime>
}

impl Scheduler {
    fn new(operations: Vec<Operation>) -> Scheduler {
        Scheduler {
            operations,
            schedule: Arc::new(Mutex::new(HashMap::new())),
            runtime: Arc::new(Runtime::new().unwrap()),
        }
    }

    fn initial_scheduling(&mut self) {
        for operation in &self.operations {
            Self::schedule_operation(
                Arc::clone(&self.runtime),
                Arc::clone(&self.schedule),
                operation.clone()
            )
        }
    }

    fn schedule_operation(runtime: Arc<Runtime>,
                          schedule: Schedule,
                          operation: Operation) {
        let (shutdown_tx, mut shutdown_tr) = watch::channel(());
        schedule.lock().unwrap().insert(operation.id, shutdown_tx);
        let duration = WeekTime::now().interval(&operation.weektime);

        Self::schedule_operation_constructor(
            Arc::clone(&runtime),
            Arc::clone(&schedule),
            duration,
            operation,
            shutdown_tr
        );
    }

    fn schedule_operation_constructor(runtime: Arc<Runtime>,
                                      schedule: Schedule,
                                      duration: Duration,
                                      operation: Operation,
                                      mut shutdown_tr: watch::Receiver<()>) {
        let fun = operation.operation;
        let rt2 = Arc::clone(&runtime);
        runtime.as_ref().spawn(
            async move {
                tokio::select! {
                        _ = shutdown_tr.changed() => {
                            Self::schedule_operation(
                                rt2,
                                Arc::clone(&schedule),
                                operation
                                );
                        }
                        _ = tokio::time::sleep(duration) => {(fun)();}
                    }
            }
        );
    }
}

fn main() {
    let mut scheduler = Scheduler::new(vec![
        Operation {
            id: 0,
            weektime: WeekTime::now().add_duration(&Duration::from_secs(3)),
            operation: &|| println!("3 seconds passed"),
        },
        Operation {
            id: 1,
            weektime: WeekTime::now().add_duration(&Duration::from_secs(4)),
            operation: &|| println!("4 seconds passed"),
        },
        Operation {
            id: 2,
            weektime: WeekTime::now().add_duration(&Duration::from_secs(5)),
            operation: &|| println!("5 seconds passed"),
        },
        Operation {
            id: 3,
            weektime: WeekTime::now().add_duration(&Duration::from_secs(6)),
            operation: &|| println!("6 seconds passed"),
        },
        Operation {
            id: 4,
            weektime: WeekTime::now().add_duration(&Duration::from_secs(7)),
            operation: &|| println!("7 seconds passed"),
        },
        Operation {
            id: 5,
            weektime: WeekTime::now().add_duration(&Duration::from_secs(8)),
            operation: &|| println!("8 seconds passed"),
        },
        Operation {
            id: 6,
            weektime: WeekTime::now().add_duration(&Duration::from_secs(9)),
            operation: &|| println!("9 seconds passed"),
        },
        Operation {
            id: 7,
            weektime: WeekTime::now().add_duration(&Duration::from_secs(10)),
            operation: &|| println!("10 seconds passed"),
        },
    ]);

    scheduler.initial_scheduling();

    thread::sleep(Duration::from_secs(5));

    drop(scheduler);

    // thread::sleep(Duration::from_secs(8));
}
