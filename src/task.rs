use chrono::{DateTime, Local};
use teloxide::requests::Requester;
use teloxide::types::ChatId;
use teloxide::Bot;
use tokio::select;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum TaskCommand {
    Enable,
    Disable,
    Delete,
}

// pub type Notify = Arc<Mutex<dyn Notification>>;

pub trait Notification: Send + Sync + 'static {
    fn preping(&self) -> Option<u16>;
    fn enabled(&self) -> bool;
    fn enable(&mut self);
    fn disable(&mut self);
    fn next_ping(&self) -> Option<DateTime<Local>>;
    fn message(&self) -> String;
}

pub fn create_task(notify: impl Notification, bot: Bot, chat_id: ChatId) -> mpsc::Sender<TaskCommand> {
    // let notify = notify;
    let (control_tx, mut control_rx) = mpsc::channel::<TaskCommand>(1);

    tokio::spawn(async move {
        // let notilock = notify.lock().await;
        loop {
            if notify.enabled() {
                let now = chrono::offset::Local::now();
                let waiter = tokio::time::sleep((notify.next_ping().unwrap() - now).to_std().unwrap());

                select! {
                    _ = waiter => {
                        println!("got ping on: \'{}\'", notify.message());
                        bot.send_message(chat_id, notify.message()).await.unwrap();
                        break;
                    },
                    tc = control_rx.recv() => {println!("got task command {tc:#?}"); todo!("commands for tasks")}
                }
            };
            //
        }
    });

    control_tx
}
