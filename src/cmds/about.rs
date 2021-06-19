// Show information about the project

use locales::t;

pub struct About {}
impl About { pub const fn new() -> Self { Self {} } }

#[async_trait::async_trait]
impl crate::cmds::Command for About {
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
        let result = ctx.room.unwrap().send(matrix_sdk::events::AnyMessageEventContent::RoomMessage(matrix_sdk::events::room::message::MessageEventContent::notice_html(
            t!("cmd.about.plain_response", description: ctx.info["DESCRIPTION"], source: ctx.info["REPOSITORY"], version: ctx.info["VERSION"], ctx.lang),
            t!("cmd.about.html_response", description: ctx.info["DESCRIPTION"], source: ctx.info["REPOSITORY"], version: ctx.info["VERSION"], ctx.lang),
        )), None).await;

        match result {
            Ok(_) => {},
            Err(msg) => ctx.logger.log(crate::log::LogLevel::Error, t!("err.matrix.event_send", err: &msg.to_string(), ctx.lang)),
        }
    }
}