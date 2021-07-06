/*

MxSelfBot by 0x1a8510f2

Module for keeping track of and invoking individual commands that can be executed

*/

use locales::t;
use crate::log::LogLevel;

// All commands which can be executed by this bot must implement this trait
#[async_trait::async_trait]
pub trait Command: Send + Sync {
    fn help(&self, short: bool) -> [String; 2];
    async fn handle(
        &self,
        gctx: crate::context::GlobalCtx,
        ectx: crate::context::EventCtx,
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
pub async fn execute(gctx: crate::context::GlobalCtx, ectx: crate::context::EventCtx) {

    let cmdline: Vec<String> = ectx.body[gctx.command_prefix.len()..].split(" ").collect::<Vec<&str>>().iter().map(|s| String::from(*s)).collect();
    let mut respond_result = None;

    if AVAIL_CMDS.contains_key(&*cmdline[0]) {

        // If the command exists, run it and return
        AVAIL_CMDS[&*cmdline[0]].handle(
            gctx.clone(),
            ectx.clone(),
        ).await;

    } else if cmdline[0] == "help" {
        // The help command is special since it needs to consider AVAIL_CMDS - hence it is hardcoded here

        // Check if any subcommand is specified
        if cmdline.len() > 1 {
            // If yes and it exists
            if AVAIL_CMDS.contains_key(&*cmdline[1]) {
                let help_subject = &*cmdline[1].to_string();
                let help_message = AVAIL_CMDS[&*cmdline[1]].help(false);

                respond_result = Some(ectx.room.send(matrix_sdk::ruma::events::AnyMessageEventContent::RoomMessage(matrix_sdk::ruma::events::room::message::MessageEventContent::notice_html(
                    t!("cmd.help.specific_plain", cmd: &help_subject, help: &help_message[0], gctx.lang),
                    t!("cmd.help.specific_html", cmd: &help_subject, help: &help_message[1], gctx.lang),
                )), None).await);
            } else {
                respond_result = Some(ectx.room.send(matrix_sdk::ruma::events::AnyMessageEventContent::RoomMessage(matrix_sdk::ruma::events::room::message::MessageEventContent::notice_plain(
                    "B",
                )), None).await);
            }
        } else {
            respond_result = Some(ectx.room.send(matrix_sdk::ruma::events::AnyMessageEventContent::RoomMessage(matrix_sdk::ruma::events::room::message::MessageEventContent::notice_plain(
                "Soon™",
            )), None).await);
        }
    } else {
        // If none of the above matched, the command is not recognised
        respond_result = Some(ectx.room.send(matrix_sdk::ruma::events::AnyMessageEventContent::RoomMessage(matrix_sdk::ruma::events::room::message::MessageEventContent::notice_plain(
            t!("err.cmd.unknown_cmd", cmd: &cmdline[0], gctx.lang),
        )), None).await);
    }

    match respond_result {
        None => {},
        Some(result) => {
            match result {
                Ok(_) => {},
                Err(msg) => gctx.logger.log(LogLevel::Error, t!("err.matrix.event_send", err: &msg.to_string(), gctx.lang)),
            }
        }
    }
}

//