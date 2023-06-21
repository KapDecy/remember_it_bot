use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime};
use derive_builder::Builder;
use teloxide::prelude::*;

use crate::task::Notification;
use crate::{HandlerResult, MyDialogue, State};

#[derive(Debug, Builder, Clone)]
pub struct SimpleNotification {
    enabled: bool,
    text: String,
    date: NaiveDate,
    daytime: NaiveTime,
}

impl Notification for SimpleNotification {
    fn preping(&self) -> Option<u16> {
        None
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false;
    }

    fn next_ping(&self) -> Option<chrono::DateTime<Local>> {
        let off = FixedOffset::east_opt(3 * 3600).unwrap(); // MOSCOW = UTC+3
        match self.enabled {
            // true => Some(DateTime::from(NaiveDateTime::new(self.date, self.daytime), off)),
            true => Some(DateTime::from_local(NaiveDateTime::new(self.date, self.daytime), off)),
            false => None,
        }
    }

    fn message(&self) -> String {
        self.text.clone()
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone)]
pub enum SimpleNotificationBuildState {
    Text,
    Date(SimpleNotificationBuilder),
    Time(SimpleNotificationBuilder),
    Build(SimpleNotificationBuilder),
}

pub(crate) async fn text(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    let mut snb = SimpleNotificationBuilder::create_empty();
    if let Some(text) = msg.text() {
        snb.text(text.to_string());
        dialogue
            .update(State::SimpleNotificationBuild(SimpleNotificationBuildState::Date(snb)))
            .await?;
        bot.send_message(
            msg.chat.id,
            "When you want me to send you this notification? \n\"dd:mm:yyyy\"",
        )
        .await?;
    }

    Ok(())
}

pub(crate) async fn date(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    mut snb: SimpleNotificationBuilder,
) -> HandlerResult {
    if let Some(text) = msg.text() {
        let v: Vec<_> = text.split(':').map(|x| x.parse::<i32>().unwrap()).collect();
        let date = NaiveDate::from_ymd_opt(v[2], v[1] as u32, v[0] as u32).unwrap();
        snb.date(date);

        dialogue
            .update(State::SimpleNotificationBuild(SimpleNotificationBuildState::Time(snb)))
            .await?;

        bot.send_message(
            msg.chat.id,
            "What time of day you want me to send you this notification? \n\"hh:mm\"",
        )
        .await?;
    }
    Ok(())
}
pub(crate) async fn time(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    mut snb: SimpleNotificationBuilder,
) -> HandlerResult {
    if let Some(text) = msg.text() {
        let v: Vec<_> = text.split(':').map(|x| x.parse::<u32>().unwrap()).collect();
        let time = NaiveTime::from_hms_opt(v[0], v[1], 0).unwrap();
        snb.daytime(time);

        dialogue
            .update(State::SimpleNotificationBuild(SimpleNotificationBuildState::Build(snb)))
            .await?;

        bot.send_message(
            msg.chat.id,
            "What time of day you want me to send you this notification? \n\"hh:mm\"",
        )
        .await?;
    }
    Ok(())
}
pub(crate) async fn build(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    snb: SimpleNotificationBuilder,
) -> HandlerResult {
    todo!();
    Ok(())
}
