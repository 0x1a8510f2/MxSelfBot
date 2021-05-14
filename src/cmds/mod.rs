mod ping;

#[async_trait::async_trait]
pub trait Command: Send + Sync {
    fn help(&self, long: bool) -> String;
    async fn handle(
        &self,
        cmdline: &Vec<&str>,
        event: &matrix_sdk::events::SyncMessageEvent<matrix_sdk::events::room::message::MessageEventContent>,
        room: &matrix_sdk::room::Joined,
    );
}

lazy_static::lazy_static! {
    static ref COMMANDS: std::collections::HashMap<&'static str, Box<dyn Command>> = {
        let mut m = std::collections::HashMap::new();
        m.insert("ping", Box::new(ping::Ping::new()) as Box<dyn Command>);
        m
    };
}

pub async fn execute(
    cmdline: &Vec<&str>,
    event: &matrix_sdk::events::SyncMessageEvent<matrix_sdk::events::room::message::MessageEventContent>,
    room: &matrix_sdk::room::Joined,
) {
    if COMMANDS.contains_key(cmdline[0]) {
        COMMANDS[cmdline[0]].handle(cmdline, event, room).await;
    }
    /*match cmdline[0] {
        "ping" => {
            ping::Ping::new().handle(cmdline, event, room).await;
        }
        _ => {
            // If the command did not match anything above, it's invalid and the user should be alerted of that
            let content = matrix_sdk::events::AnyMessageEventContent::RoomMessage(matrix_sdk::events::room::message::MessageEventContent::notice_plain(
                ""
            ));

            room.send(content, None).await;
        }
    }*/
}