// All commands which can be executed by this bot must implement this trait
#[async_trait::async_trait]
pub trait Command: Send + Sync {
    fn help(&self, short: bool) -> String;
    async fn handle(
        &self,
        ctx: crate::context::Ctx,
    ) -> Option<matrix_sdk::events::AnyMessageEventContent>;
}

// Define a list of all available commands - this allows for easily generating help messages while avoiding
// code duplication.
mod ping;
mod info;
lazy_static::lazy_static! {
    static ref AVAIL_CMDS: std::collections::HashMap<&'static str, Box<dyn Command>> = {
        let mut m = std::collections::HashMap::new();
        m.insert("ping", Box::new(ping::Ping::new()) as Box<dyn Command>);
        m.insert("info", Box::new(info::Info::new()) as Box<dyn Command>);
        m
    };
}

// Given the commandline, execute the correct command and return its results
pub async fn execute(
    ctx: crate::context::Ctx,
) -> Option<matrix_sdk::events::AnyMessageEventContent> {
    if AVAIL_CMDS.contains_key(&*ctx.cmdline[0]) {
        // If the command exists, run it and return its result
        return AVAIL_CMDS[&*ctx.cmdline[0]].handle(ctx).await
    } else if ctx.cmdline[0] == "help" {
        // The help command is special since it needs to consider AVAIL_CMDS - hence it is hardcoded here
        return Option::Some(matrix_sdk::events::AnyMessageEventContent::RoomMessage(matrix_sdk::events::room::message::MessageEventContent::notice_plain(
            "Soon™",
        )))
    }
    // If none of the above matched, the command is not recognised
    Option::Some(matrix_sdk::events::AnyMessageEventContent::RoomMessage(matrix_sdk::events::room::message::MessageEventContent::notice_plain(
        format!("The command `{}` was not recognised. Try using the `help [command_name]` command to get a list of available commands or information about a specific command.", ctx.cmdline[0]),
    )))
}