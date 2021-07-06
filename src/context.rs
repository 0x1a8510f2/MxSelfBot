/*

MxSelfBot by 0x1a8510f2

Contexts passed around within the bot which should contain all information
necessary for any functions of the bot.

*/

// Properties which are shared throughout the entire program
// If they are changed in one place, they should be changed everywhere
#[derive(Clone)]
pub struct GlobalCtx {
    pub info: std::collections::HashMap<&'static str, &'static str>,
    pub hs_url: String,
    pub username: String,
    pub command_prefix: String,
    pub lang: String,
    pub logger: crate::Logger,
}
impl GlobalCtx {
    pub fn new(
        info: std::collections::HashMap<&'static str, &'static str>,
        hs_url: String,
        username: String,
        command_prefix: String,
        lang: String,
        logger: crate::Logger,
    ) -> Self {
        Self {
            info,
            hs_url,
            username,
            command_prefix,
            lang,
            logger,
        }
    }
}

// Properties which are unique to each message received
#[derive(Clone)]
pub struct EventCtx {
    pub body: String,
    pub sender: String,
    pub room: matrix_sdk::room::Joined,
}
impl EventCtx {
    pub fn new(
        body: String,
        sender: String,
        room: matrix_sdk::room::Joined,
    ) -> Self {
        Self {
            body,
            sender,
            room,
        }
    }
}