// Check if the bot is online

pub struct Ping {}
impl Ping { pub const fn new() -> Self { Self {} } }

#[async_trait::async_trait]
impl crate::cmds::Command for Ping {
    fn help(&self, short: bool) -> String {
        match short {
            true => "".to_string(),
            false => "".to_string(),
        }
    }

    async fn handle(
        &self,
        ctx: crate::context::Ctx,
    ) {
        let result = ctx.room.unwrap().send(matrix_sdk::events::AnyMessageEventContent::RoomMessage(matrix_sdk::events::room::message::MessageEventContent::notice_plain(
            "Pong 🏓",
        )), None).await;
    }
}