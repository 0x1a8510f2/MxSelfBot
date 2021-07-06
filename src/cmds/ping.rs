// Check if the bot is online

use locales::t;

pub struct Ping {}
impl Ping { pub const fn new() -> Self { Self {} } }

#[async_trait::async_trait]
impl crate::cmds::Command for Ping {
    fn help(&self, short: bool) -> [String; 2] {
        match short {
            true => [
                "Check if the bot is online. The bot should reply with \"Pong 🏓\".".to_string(),
                "Check if the bot is online. The bot should reply with \"Pong 🏓\".".to_string(),
            ],
            false => ["long".to_string(), "".to_string()],
        }
    }

    async fn handle(
        &self,
        ctx: crate::context::Ctx,
    ) {
        let result = ctx.room.unwrap().send(matrix_sdk::ruma::events::AnyMessageEventContent::RoomMessage(matrix_sdk::ruma::events::room::message::MessageEventContent::notice_plain(
            "Pong 🏓",
        )), None).await;

        match result {
            Ok(_) => {},
            Err(msg) => ctx.logger.log(crate::log::LogLevel::Error, t!("err.matrix.event_send", err: &msg.to_string(), ctx.lang)),
        }
    }
}