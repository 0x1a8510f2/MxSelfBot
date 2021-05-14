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
        _cmdline: &Vec<&str>,
        _event: &matrix_sdk::events::SyncMessageEvent<matrix_sdk::events::room::message::MessageEventContent>,
        _room: &matrix_sdk::room::Joined,
        _bot: &crate::MxSelfBot,
    ) -> Option<matrix_sdk::events::AnyMessageEventContent> {
        Option::Some(matrix_sdk::events::AnyMessageEventContent::RoomMessage(matrix_sdk::events::room::message::MessageEventContent::notice_plain(
            "Pong 🏓",
        )))
    }
}