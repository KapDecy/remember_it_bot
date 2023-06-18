use std::error::Error;

use chrono::NaiveTime;
use derive_builder::Builder;
use teloxide::types::Me;
use teloxide::utils::command::BotCommands;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub enum RememberUnit {
    Birthday(Birthday),
}
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

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    AddBirthday,
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    BirthdayBuild(BirthdayBuildState),
}

#[derive(Clone)]
pub enum BirthdayBuildState {
    Name,
    Date(BirthdayBuilder),
    Preping(BirthdayBuilder),
    DaytimeToPing(BirthdayBuilder),
    Build(BirthdayBuilder),
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let bot = Bot::from_env();

    let handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(dptree::case![State::Start].endpoint(command_handler))
        .branch(
            dptree::case![State::BirthdayBuild(bbs)]
                .branch(dptree::case![BirthdayBuildState::Name].endpoint(birthday::name))
                .branch(dptree::case![BirthdayBuildState::Date(bb)].endpoint(birthday::date))
                .branch(dptree::case![BirthdayBuildState::Preping(bb)].endpoint(birthday::preping))
                .branch(
                    dptree::case![BirthdayBuildState::DaytimeToPing(bb)]
                        .endpoint(birthday::daytime),
                ),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

pub mod birthday {
    use std::error::Error;

    use teloxide::prelude::*;
    use teloxide::types::ParseMode;

    use crate::{BirthdayBuildState, BirthdayBuilder, MyDialogue, State};

    pub(crate) async fn name(
        bot: Bot,
        msg: Message,
        dialogue: MyDialogue,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match msg.text() {
            Some(text) => {
                let mut bb = BirthdayBuilder::create_empty();
                bb.name(text.into());
                dialogue
                    .update(State::BirthdayBuild(BirthdayBuildState::Date(bb)))
                    .await?;
                bot.send_message(msg.chat.id, "What's the date?\n Enter in dd:mm:yyyy or dd:mm if you don't know year ||shame on you||".to_string()).parse_mode(ParseMode::MarkdownV2).await?;
            }
            None => todo!(),
        }

        Ok(())
    }

    pub(crate) async fn date(
        bot: Bot,
        msg: Message,
        dialogue: MyDialogue,
        mut bb: BirthdayBuilder,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    pub(crate) async fn preping(
        bot: Bot,
        msg: Message,
        dialogue: MyDialogue,
        mut bb: BirthdayBuilder,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    pub(crate) async fn daytime(
        bot: Bot,
        msg: Message,
        dialogue: MyDialogue,
        mut bb: BirthdayBuilder,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match msg.text() {
            Some(text) => {
                let (hh, mm): (u32, u32) = text
                    .split_once(':')
                    .map(|(h, m)| (h.parse().unwrap(), m.parse().unwrap()))
                    .unwrap();
                let dt = chrono::NaiveTime::from_hms_opt(hh, mm, 0).unwrap();
                bb.daytime_to_ping(dt);
                bot.send_message(
                    msg.chat.id,
                    "Nice! Now let's check what we got.".to_string(),
                )
                .await?;
                bot.send_message(msg.chat.id, String::from(format!("name: {:?}", bb.name)))
                    .await?;
                bot.send_message(
                    msg.chat.id,
                    String::from(format!(
                        "birthday: {:?}:{:?}:{:?}",
                        bb.bday, bb.bmonth, bb.byear
                    )),
                )
                .await?;
                bot.send_message(
                    msg.chat.id,
                    String::from(format!("preping: {:?}", bb.preping)),
                )
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
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            // Ok(Command::Help) => {
            //     // Just send the description of all commands.
            //     // bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
            // }
            Ok(Command::AddBirthday) => {
                dialogue
                    .update(State::BirthdayBuild(BirthdayBuildState::Name))
                    .await?;
                bot.send_message(msg.chat.id, "name".to_string()).await?;

                // Create a list of buttons and send them.
                // let keyboard = make_keyboard();
                // bot.send_message(msg.chat.id, "Debian versions:").reply_markup(keyboard).await?;
            }

            Err(_) => {
                bot.send_message(msg.chat.id, "Command not found!").await?;
            }
        }
    }

    Ok(())
}
