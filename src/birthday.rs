use std::error::Error;

use chrono::NaiveTime;
use derive_builder::Builder;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::{HandlerResult, MyDialogue, State};

#[derive(Debug, Builder, Clone)]
pub struct Birthday {
    enabled: bool,
    name: String,
    bday: u8,
    bmonth: u8,
    byear: Option<u16>,
    // How many days before birthday should be first ping
    preping: Option<u8>,
    daytime_to_ping: NaiveTime,
}

#[derive(Clone)]
pub enum BirthdayBuildState {
    Name,
    Date(BirthdayBuilder),
    Preping(BirthdayBuilder),
    DaytimeToPing(BirthdayBuilder),
    Build(BirthdayBuilder),
}

pub(crate) async fn name(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let mut bb = BirthdayBuilder::create_empty();
            bb.name(text.into());
            dialogue
                .update(State::BirthdayBuild(BirthdayBuildState::Date(bb)))
                .await?;
            bot.send_message(
                msg.chat.id,
                "What's the date?\n Enter in dd:mm:yyyy or dd:mm if you don't know year ||shame on you||".to_string(),
            )
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        }
        None => todo!(),
    }

    Ok(())
}

pub(crate) async fn date(bot: Bot, msg: Message, dialogue: MyDialogue, mut bb: BirthdayBuilder) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let v: Vec<u16> = text.split(':').map(|x| x.parse::<u16>().unwrap()).collect();
            match v.len() {
                2 => {
                    bot.send_message(msg.chat.id, "||shame on you||".to_string())
                        .parse_mode(ParseMode::MarkdownV2)
                        .await?;
                    bb.bday(v[0] as u8);
                    bb.bmonth(v[1] as u8);
                    dialogue
                        .update(State::BirthdayBuild(BirthdayBuildState::Preping(bb)))
                        .await?;
                    bot.send_message(
                            msg.chat.id,
                            "If you would like me to remind you of this birthday in advance, specify how many days in advance you would like a reminder (0 if you don't)"
                                .to_string(),
                        )
                        .await?;
                }
                3 => {
                    bb.bday(v[0] as u8);
                    bb.bmonth(v[1] as u8);
                    bb.byear(Some(v[2]));
                    dialogue
                        .update(State::BirthdayBuild(BirthdayBuildState::Preping(bb)))
                        .await?;
                    bot.send_message(
                            msg.chat.id,
                            "If you would like me to remind you of this birthday in advance, specify how many days in advance you would like a reminder (0 if you don't)"
                                .to_string(),
                        )
                        .await?;
                }
                _ => {
                    todo!()
                }
            }
        }
        None => todo!(),
    }
    // bb.name(text.into());

    Ok(())
}

pub(crate) async fn preping(bot: Bot, msg: Message, dialogue: MyDialogue, mut bb: BirthdayBuilder) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            if let Ok(n) = text.parse::<u8>() {
                if n == 0 {
                    bb.preping(None);
                } else {
                    bb.preping(Some(n));
                }
                dialogue
                    .update(State::BirthdayBuild(BirthdayBuildState::DaytimeToPing(bb)))
                    .await?;
                bot.send_message(
                    msg.chat.id,
                    "What time of a day you want me to notify you?\n format: hh:mm".to_string(),
                )
                .await?;
            }
        }
        None => todo!(),
    }
    Ok(())
}

pub(crate) async fn daytime(bot: Bot, msg: Message, dialogue: MyDialogue, mut bb: BirthdayBuilder) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let (hh, mm): (u32, u32) = text
                .split_once(':')
                .map(|(h, m)| (h.parse().unwrap(), m.parse().unwrap()))
                .unwrap();
            let dt = chrono::NaiveTime::from_hms_opt(hh, mm, 0).unwrap();
            bb.daytime_to_ping(dt);
            bot.send_message(msg.chat.id, "Nice! Now let's check what we got.".to_string())
                .await?;
            bot.send_message(msg.chat.id, String::from(format!("name: {:?}", bb.name)))
                .await?;
            bot.send_message(
                msg.chat.id,
                String::from(format!("birthday: {:?}:{:?}:{:?}", bb.bday, bb.bmonth, bb.byear)),
            )
            .await?;
            bot.send_message(msg.chat.id, String::from(format!("preping: {:?}", bb.preping)))
                .await?;
            bot.send_message(
                msg.chat.id,
                String::from(format!("Daytime to ping: {:?}", bb.daytime_to_ping)),
            )
            .await?;
            dialogue
                .update(State::BirthdayBuild(BirthdayBuildState::Build(bb)))
                .await?;
        }
        None => todo!(),
    }
    Ok(())
}
