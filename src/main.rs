mod birthday;
mod simple_notification;
mod task;

use std::collections::HashMap;

use birthday::BirthdayBuildState;
use simple_notification::SimpleNotificationBuildState;
use teloxide::types::Me;
use teloxide::utils::command::BotCommands;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

// pub enum RememberUnit {
//     Birthday(Birthday),
// }

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    Help,
    AddBirthday,
    SimpleNotification,
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    BirthdayBuild(BirthdayBuildState),
    SimpleNotificationBuild(simple_notification::SimpleNotificationBuildState),
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let token = dotenvy::var("TELOXIDE_TOKEN").unwrap();

    // let bot = Bot::from_env();
    let bot = Bot::new(token);

    let notifys: HashMap<String, tokio::sync::mpsc::Sender<task::TaskCommand>> = HashMap::new();

    let handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(dptree::case![State::Start].endpoint(command_handler))
        .branch(
            dptree::case![State::BirthdayBuild(bbs)]
                .branch(dptree::case![BirthdayBuildState::Name].endpoint(birthday::name))
                .branch(dptree::case![BirthdayBuildState::Date(bb)].endpoint(birthday::date))
                .branch(dptree::case![BirthdayBuildState::Preping(bb)].endpoint(birthday::preping))
                .branch(dptree::case![BirthdayBuildState::DaytimeToPing(bb)].endpoint(birthday::daytime)),
        )
        .branch(
            dptree::case![State::SimpleNotificationBuild(snbs)]
                .branch(dptree::case![SimpleNotificationBuildState::Text].endpoint(simple_notification::text))
                .branch(dptree::case![SimpleNotificationBuildState::Date(snb)].endpoint(simple_notification::date))
                .branch(dptree::case![SimpleNotificationBuildState::Time(snb)].endpoint(simple_notification::time))
                .branch(dptree::case![SimpleNotificationBuildState::Build(snb)].endpoint(simple_notification::build)),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn command_handler(bot: Bot, msg: Message, dialogue: MyDialogue, me: Me) -> HandlerResult {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                // Just send the description of all commands.
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }
            Ok(Command::SimpleNotification) => {
                dialogue
                    .update(State::SimpleNotificationBuild(SimpleNotificationBuildState::Text))
                    .await?;
                bot.send_message(msg.chat.id, "Text of notification?".to_string())
                    .await?;
            }

            Ok(Command::AddBirthday) => {
                dialogue.update(State::BirthdayBuild(BirthdayBuildState::Name)).await?;
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
