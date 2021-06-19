/*

MxSelfBot by 0x1a8510f2

Module for keeping track of and invoking individual commands that can be executed

*/

use locales::t;
use crate::log::LogLevel;

// All commands which can be executed by this bot must implement this trait
#[async_trait::async_trait]
pub trait Command: Send + Sync {
    fn help(&self, short: bool) -> String;
    async fn handle(
        &self,
        ctx: crate::context::Ctx,
    );
}

// Define a list of all available commands - this allows for easily generating help messages while avoiding
// code duplication.
mod about;
mod ping;
lazy_static::lazy_static! {
    static ref AVAIL_CMDS: std::collections::HashMap<&'static str, Box<dyn Command>> = {
        let mut m = std::collections::HashMap::new();
        m.insert("about", Box::new(about::About::new()) as Box<dyn Command>);
        m.insert("ping", Box::new(ping::Ping::new()) as Box<dyn Command>);
        m
    };
}

// Given the commandline (within context), execute the correct command and return its results
pub async fn execute(ctx: crate::context::Ctx) {
    if AVAIL_CMDS.contains_key(&*ctx.cmdline[0]) {
        // If the command exists, run it and return
        AVAIL_CMDS[&*ctx.cmdline[0]].handle(ctx).await;
    } else if ctx.cmdline[0] == "help" {
        // The help command is special since it needs to consider AVAIL_CMDS - hence it is hardcoded here
        let result = ctx.room.unwrap().send(matrix_sdk::events::AnyMessageEventContent::RoomMessage(matrix_sdk::events::room::message::MessageEventContent::notice_plain(
            "Soon™",
        )), None).await;

        match result {
            Ok(_) => {},
            Err(msg) => ctx.logger.log(LogLevel::Error, t!("err.matrix.event_send", err: &msg.to_string(), ctx.lang)),
        }
    } else {
        // If none of the above matched, the command is not recognised
        let result = ctx.room.unwrap().send(matrix_sdk::events::AnyMessageEventContent::RoomMessage(matrix_sdk::events::room::message::MessageEventContent::notice_plain(
            t!("err.cmd.unknown_cmd", cmd: &ctx.cmdline[0], ctx.lang),
        )), None).await;

        match result {
            Ok(_) => {},
            Err(msg) => ctx.logger.log(LogLevel::Error, t!("err.matrix.event_send", err: &msg.to_string(), ctx.lang)),
        }
    }
}

//