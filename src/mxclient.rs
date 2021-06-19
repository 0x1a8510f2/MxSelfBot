/*

MxSelfBot by 0x1a8510f2

Functions helping with interactions with the Matrix network

*/

// Bring some stuff into the scope
use crate::log::LogLevel;
use crate::cmds;
use url;
use matrix_sdk::{
    self, async_trait,
    events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        SyncMessageEvent,
    },
    room::Room,
    Client, ClientConfig, EventHandler, SyncSettings,
};
use locales::t;

// Authenticate with the homeserver and return an instance of matrix_sdk::Client
pub async fn connect(hs_url: &str, username: &str, password: &str, device_name: &str)
    -> Result<matrix_sdk::Client, Box<dyn std::error::Error>> {

    let client_config = ClientConfig::new();
    let hs_url_parsed = url::Url::parse(&hs_url)?;
    let client = Client::new_with_config(hs_url_parsed, client_config)?;

    client.login(&username, &password, None, Some(&device_name)).await?;

    Ok(client)
}

// End session with homeserver
pub async fn disconnect(_client: &matrix_sdk::Client) -> Result<(), String> {
    // TODO
    // Waiting on https://github.com/matrix-org/matrix-rust-sdk/issues/115

    /*if self.client.logged_in().await {
    }*/
    Ok(())
}

// Run the sync loop and handle events until kill_loop is toggled
pub async fn run(client: &matrix_sdk::Client, eh: Box<MxSelfBotEventHandler>, kill_loop: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Result<(), matrix_sdk::Error> {
    // Run an initial sync so the bot doesn't respond to old messages
    client.sync_once(SyncSettings::default()).await.unwrap();

    // Add MxSelfBotEventHandler to be notified of incoming messages such that they can be handled
    client.set_event_handler(eh).await;

    // Since we called sync_once before entering the sync loop we must pass
    // that sync token to sync
    let settings = SyncSettings::default().token(client.sync_token().await.unwrap());

    // Sync until the exit flag changes
    client.sync_with_callback(settings, |_| async {
        if kill_loop.load(std::sync::atomic::Ordering::Relaxed) {
            matrix_sdk::LoopCtrl::Break
        } else {
            matrix_sdk::LoopCtrl::Continue
        }
    }).await;

    Ok(())
}

// The event handler responsible for processing incoming events
pub struct MxSelfBotEventHandler { ctx: crate::context::Ctx }
impl MxSelfBotEventHandler { pub fn new(
    info: std::collections::HashMap<&'static str, &'static str>,
    username: String,
    command_prefix: String,
    lang: String,
    logger: crate::Logger,
) -> Self {
    let ctx = crate::context::Ctx::new(
        info,
        username,
        command_prefix,
        lang,
        logger,
        Vec::new(),
        None,
        String::new(),
    );
    Self { ctx }
} }

// The logic behind MxSelfBotEventHandler
#[async_trait]
impl EventHandler for MxSelfBotEventHandler {
    // Handle message events in any room the user is in
    async fn on_room_message(&self, room: Room, event: &SyncMessageEvent<MessageEventContent>) {
        if let Room::Joined(room) = room {
            // Extract needed data from the received event
            let (msg_body, msg_sender) = if let SyncMessageEvent {
                sender: msg_sender,
                content:
                    MessageEventContent {
                        msgtype: MessageType::Text(TextMessageEventContent { body: msg_body, .. }),
                        ..
                    },
                ..
            } = event
            { (msg_body, msg_sender) }
            else { return; };

            // Only process messages as commands if they start with the prefix and *are sent by our account* (very important)
            if msg_body.starts_with(&self.ctx.command_prefix) && msg_sender.to_string() == self.ctx.username {
                // Create a copy of the context with command-specific options filled in
                let mut tmp_ctx = self.ctx.clone();
                // This removes the command prefix, then splits the remaining string by spaces and converts the result into a Vec<String>
                tmp_ctx.cmdline = msg_body[self.ctx.command_prefix.len()..].split(" ").collect::<Vec<&str>>().iter().map(|s| String::from(*s)).collect();
                tmp_ctx.room = Some(room.clone());
                tmp_ctx.sender = msg_sender.to_string();

                tmp_ctx.logger.log(LogLevel::Info,
                    t!("info.command.recv", cmdline: &tmp_ctx.cmdline.join(" "), room_id: room.room_id().as_str(), sender: msg_sender.as_str(), tmp_ctx.lang));

                // Execute the command with the newly-created context
                cmds::execute(tmp_ctx).await;

            // If the event was not processed as a command, we may still need to consider it (autoreply, for instance)
            } else {

                // TODO

            }
        }
    }
}