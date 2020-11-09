use tokio::sync::mpsc::{Sender, Receiver};
use tokio::time::Duration;
use weektime::WeekTime;
use std::error::Error;
use std::fmt::{self, Debug, Formatter};
use chrono::Local;
use env_logger::{Builder};
use log::{debug};
use std::io::Write;
use std::sync::Arc;

pub mod weektime;

///# Example:
///
///#[tokio::main]
///async fn main() -> Result<(), Box<dyn Error>>{
///    init_logging();
///    let b = Box::new(2);
///    let scheduler = Scheduler::new();
///    scheduler.initial_scheduling(vec![
///        Operation {
///            time: WeekTime::from_seconds(WeekTime::now().to_seconds() + 3),
///            operation: Box::new(move || println!("3 seconds passed {}", b)),
///        },
///        Operation {
///            time: WeekTime::from_seconds(WeekTime::now().to_seconds() + 4),
///            operation: Box::new(|| println!("4 seconds passed")),
///        },
///        Operation {
///            time: WeekTime::from_seconds(WeekTime::now().to_seconds() + 5),
///            operation: Box::new(|| println!("5 seconds passed")),
///        },
///        Operation {
///            time: WeekTime::from_seconds(WeekTime::now().to_seconds() + 6),
///            operation: Box::new(|| println!("6 seconds passed")),
///        },
///        Operation {
///            time: WeekTime::from_seconds(WeekTime::now().to_seconds() + 7),
///            operation: Box::new(|| println!("7 seconds passed")),
///        }
///    ]).await?;
///
///    std::thread::sleep(std::time::Duration::from_secs(3));
///
///    std::thread::sleep(std::time::Duration::from_secs(5));
///
///    Ok(())
///}


pub struct Operation {
    time: WeekTime,
    operation: Box<dyn Fn() + Send + Sync>,
}

impl Operation {
    pub fn new(time: WeekTime, operation: Box<dyn Fn() + Send + Sync>) -> Self {
        Operation{time, operation}
    }
}

impl  Debug for Operation
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Operation {{ {:?} }}", self.time)
    }
}

pub struct Scheduler
{
    sender: Sender<Arc<Operation>>
}

impl Scheduler
{
    pub async fn new() -> Scheduler {
        let (tx, rx) = tokio::sync::mpsc::channel::<Arc<Operation>>(1000);
        tokio::spawn(Self::start_scheduler(tx.clone(), rx));

        Scheduler {
            sender: tx
        }
    }

    /// Starts the loop of the scheduler
    /// It listens on the Operation channel for Operations
    ///
    pub async fn start_scheduler(tx: Sender<Arc<Operation>>, mut rx: Receiver<Arc<Operation>>, ) -> Option<()>
    {
        loop {
            let operation = rx.recv().await?;
            debug!("scheduled operation: {:?}", operation);

            let tx = tx.clone();
            let op = operation.clone();
            tokio::spawn(async move {
                let delay = op.time.interval_from_now();
                debug!("setting to sleep for: {:#?}", delay);
                tokio::time::sleep(Duration::from_secs(4)).await; // wait till function schould be executed
                (op.operation)(); // execute scheduled function
                tx.send(op.clone()).await.unwrap(); // reschedule current operation for next week
            });
        }
    }

    pub async fn initial_scheduling(&self, operations: Vec<Operation>) -> Result<(), Box<dyn Error>> {
        for operation in operations {
            self.sender.send(Arc::new(operation)).await?;
        }
        Ok(())
    }
}

pub fn init_logging() {
    Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                     "{} [{}] - {}",
                     Local::now().format("%Y-%m-%dT%H:%M:%S"),
                     record.level(),
                     record.args()
            )
        })
        .parse_default_env()
        .init();
}

#[cfg(test)]
mod test{
    use crate::{Scheduler, Operation, init_logging};
    use crate::weektime::WeekTime;
    use std::error::Error;
    use tokio::runtime;
    use tokio::runtime::{Runtime, Builder};
    use tokio::time::Duration;

    #[test]
    fn test_rescheduling() {
        init_logging();

        let b = Box::new(2);
        let c = Box::new(3);

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.spawn(async move {
            let scheduler = Scheduler::new().await;
            scheduler.initial_scheduling(vec![
                Operation::new(
                    WeekTime::from_seconds(0),
                    Box::new(move || {println!("2: :{}", b)})),
                Operation::new(
                    WeekTime::from_seconds(0),
                    Box::new(move || {println!("3: :{}", c)})),
            ]).await;
        });
        std::thread::sleep(Duration::from_secs(15))
    }
}

