#[derive(Clone)]
pub struct Ctx {
    pub username: String,
    pub command_prefix: String,
    pub cmdline: Vec<String>,
    pub lang: String,
    pub logger: crate::Logger,
}
impl Ctx {
    pub fn new(
        username: String,
        command_prefix: String,
        cmdline: Vec<String>,
        lang: String,
        logger: crate::Logger,
    ) -> Self {
        Self {
            username,
            command_prefix,
            cmdline,
            lang,
            logger,
        }
    }
}