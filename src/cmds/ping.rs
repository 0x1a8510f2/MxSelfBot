#[derive(Clone)]
pub struct Ping {
    pub commandline_template: &'static str,
    pub description_short: &'static str,
    pub description_long: &'static str,
}
impl Ping {
    pub const fn new() -> Self {
        Self {
            commandline_template: "ping",
            description_short: "Check if the bot is reachable",
            description_long: "The ping command responds to the original message with \"Pong 🏓\" to show that the bot is active and listening for commands. It takes no arguments.",
        }
    }
}

#[async_trait::async_trait]
impl crate::cmds::Command for Ping {
    fn help(&self, long: bool) -> String {
        match long {
            true => "".to_string(),
            false => "".to_string(),
        }
    }

    async fn handle(
        &self,
        _cmdline: &Vec<&str>,
        _event: &matrix_sdk::events::SyncMessageEvent<matrix_sdk::events::room::message::MessageEventContent>,
        room: &matrix_sdk::room::Joined,
    ) {
        let content = matrix_sdk::events::AnyMessageEventContent::RoomMessage(matrix_sdk::events::room::message::MessageEventContent::notice_plain(
            "Pong 🏓",
        ));

        let _ = room.send(content, None).await;
    }
}