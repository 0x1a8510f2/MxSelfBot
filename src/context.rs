/*

MxSelfBot by 0x1a8510f2

A context passed to each command which should contain all the data required to
successfully execute the command

*/

#[derive(Clone)]
pub struct Ctx {
    // Universal properties (don't change depending on event)
    pub info: std::collections::HashMap<&'static str, &'static str>,
    pub username: String,
    pub command_prefix: String,
    pub lang: String,
    pub logger: crate::Logger,
    // Event properties (change based on event)
    pub cmdline: Vec<String>,
    pub room: Option<matrix_sdk::room::Joined>,
    pub sender: String,

}
impl Ctx {
    pub fn new(
        // up
        info: std::collections::HashMap<&'static str, &'static str>,
        username: String,
        command_prefix: String,
        lang: String,
        logger: crate::Logger,
        // ep
        cmdline: Vec<String>,
        room: Option<matrix_sdk::room::Joined>,
        sender: String,
    ) -> Self {
        Self {
            // up
            info,
            username,
            command_prefix,
            lang,
            logger,
            // ep
            cmdline,
            room,
            sender,
        }
    }
}