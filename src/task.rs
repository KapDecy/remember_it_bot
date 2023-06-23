use std::sync::Arc;

use chrono::{DateTime, Local};
use tokio::select;
use tokio::sync::{mpsc, Mutex};

#[derive(Debug)]
pub enum TaskCommand {
    Enable,
    Disable,
    Delete,
}

pub type Notify = Arc<Mutex<dyn Notification>>;

pub trait Notification: Send + 'static {
    fn preping(&self) -> Option<u16>;
    fn enabled(&self) -> bool;
    fn enable(&mut self);
    fn disable(&mut self);
    fn next_ping(&self) -> Option<DateTime<Local>>;
    fn message(&self) -> String;
}

pub fn create_task(notify: Notify) -> mpsc::Sender<TaskCommand> {
    // let notify = notify;
    let (control_tx, mut control_rx) = mpsc::channel::<TaskCommand>(1);

    tokio::spawn(async move {
        let notilock = notify.lock().await;
        loop {
            if notilock.enabled() {
                let now = chrono::offset::Local::now();
                let waiter = tokio::time::sleep((notilock.next_ping().unwrap() - now).to_std().unwrap());

                select! {
                    _ = waiter => {println!("got ping"); break;},
                    tc = control_rx.recv() => {println!("got task command {tc:#?}")}
                }
            };
            //
        }
    });

    control_tx
}
