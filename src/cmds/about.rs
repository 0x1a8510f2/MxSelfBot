// Show information about the project

use locales::t;

pub struct About {}
impl About { pub const fn new() -> Self { Self {} } }

#[async_trait::async_trait]
impl crate::cmds::Command for About {
    fn help(&self, short: bool) -> [String; 2] {
        match short {
            true => ["".to_string(), "".to_string()],
            false => ["".to_string(), "".to_string()],
        }
    }

    async fn handle(
        &self,
        gctx: crate::context::GlobalCtx,
        ectx: crate::context::EventCtx,
    ) {
        let result = ectx.room.send(matrix_sdk::ruma::events::AnyMessageEventContent::RoomMessage(matrix_sdk::ruma::events::room::message::MessageEventContent::notice_html(
            t!("cmd.about.plain_response", description: gctx.info["DESCRIPTION"], source: gctx.info["REPOSITORY"], version: gctx.info["VERSION"], gctx.lang),
            t!("cmd.about.html_response", description: gctx.info["DESCRIPTION"], source: gctx.info["REPOSITORY"], version: gctx.info["VERSION"], gctx.lang),
        )), None).await;

        match result {
            Ok(_) => {},
            Err(msg) => gctx.logger.log(crate::log::LogLevel::Error, t!("err.matrix.event_send", err: &msg.to_string(), gctx.lang)),
        }
    }
}