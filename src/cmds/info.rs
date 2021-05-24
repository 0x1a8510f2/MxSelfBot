pub struct Info {}
impl Info { pub const fn new() -> Self { Self {} } }

#[async_trait::async_trait]
impl crate::cmds::Command for Info {
    fn help(&self, short: bool) -> String {
        match short {
            true => "".to_string(),
            false => "".to_string(),
        }
    }

    async fn handle(
        &self,
        _ctx: crate::cmds::CmdCtx,
    ) -> Option<matrix_sdk::events::AnyMessageEventContent> {
        Option::Some(matrix_sdk::events::AnyMessageEventContent::RoomMessage(matrix_sdk::events::room::message::MessageEventContent::notice_html(
            format!(" | MxSelfBot v{} | \n{}\n\n Source code URL: {}", "bot.version", "bot.description", "bot.source"),
            format!(" <h1>MxSelfBot v{}</h1><i>{}</i><br/><br/>Source code URL: {}", "bot.version", "bot.description", "bot.source"),
        )))
    }
}