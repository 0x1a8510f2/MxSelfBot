// MxSelfBot by 0x1a8510f2

// Bring some stuff into the scope
use url;
use matrix_sdk::{
    self, async_trait,
    events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        AnyMessageEventContent, SyncMessageEvent,
    },
    room::Room,
    Client, ClientConfig, EventHandler, SyncSettings,
};

// The bot struct
#[derive(Clone)]
struct MxSelfBot {
    client: Client,
    username: String,
    password: String,
    homeserver_url: String,
    command_prefix: String,
}
impl MxSelfBot {
    // Create an instance of the bot with the given credentials
    // Checks the provided homeserver_url and only creates the instance if the URL is valid
    pub fn new(username: String, password: String, homeserver_url: String, command_prefix: String) -> Result<Self, url::ParseError> {

        // Create a client
        let client_config = ClientConfig::new();
        let homeserver_url_parsed = url::Url::parse(&homeserver_url)?;
        let client = Client::new_with_config(homeserver_url_parsed, client_config).unwrap();

        Ok(Self { client, username, password, homeserver_url, command_prefix })
    }

    // Authenticate with the homeserver using details provided when creating the bot instance
    pub async fn login(&self) -> Result<matrix_sdk::api::r0::session::login::Response, matrix_sdk::Error> {

        let login_result = self.client
            .login(&self.username, &self.password, None, Some("MxSelfBot"))
            .await?;

        Ok(login_result)
    }

    // End session and clean up
    pub async fn logout(&self) -> Result<(), String> {
        // Tmp
        let device_id = self.client.device_id().await;
        let device_id = match device_id {
            Some(id) => String::from(id),
            None => String::from(""),
        };
        Err(format!("Automatic logout is currently unsupported so you may need to manually logout the session with ID `{}`", device_id))

        // TODO
        /*if self.client.logged_in().await {
        }*/
    }

    // Run the sync loop and handle events
    pub async fn run(&self, kill_loop: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Result<(), matrix_sdk::Error> {
        // Run an initial sync to set up state and so the bot doesn't respond to old messages
        self.client.sync_once(SyncSettings::default()).await.unwrap();

        // Add MxSelfBotEventHandler to be notified of incoming messages, we do this after the initial
        // sync to avoid responding to messages before the bot was running
        self.client.set_event_handler(Box::new(MxSelfBotEventHandler::new(self.clone()))).await;

        // Since we called `sync_once` before we entered our sync loop we must pass
        // that sync token to `sync`
        let settings = SyncSettings::default().token(self.client.sync_token().await.unwrap());

        // Sync until the exit flag changes
        self.client.sync_with_callback(settings, |_| async {
            if kill_loop.load(std::sync::atomic::Ordering::Relaxed) {
                matrix_sdk::LoopCtrl::Break
            } else {
                matrix_sdk::LoopCtrl::Continue
            }
        }).await;

        Ok(())
    }

}

// The event handler used by MxSelfBot
struct MxSelfBotEventHandler {
    bot: MxSelfBot,
}
impl MxSelfBotEventHandler {
    fn new(bot: MxSelfBot) -> Self {
        Self { bot }
    }
}

// The logic behind MxSelfBotEventHandler
#[async_trait]
impl EventHandler for MxSelfBotEventHandler {
    // Handle message events (commands)
    async fn on_room_message(&self, room: Room, event: &SyncMessageEvent<MessageEventContent>) {
        if let Room::Joined(room) = room {
            let (msg_body, msg_sender) = if let SyncMessageEvent {
                sender: msg_sender,
                content:
                    MessageEventContent {
                        msgtype: MessageType::Text(TextMessageEventContent { body: msg_body, .. }),
                        ..
                    },
                ..
            } = event
            {
                (msg_body, msg_sender)
            } else {
                return;
            };

            // Only ever consider messages sent by our own account and which start with the command prefix
            if !(*msg_sender == self.bot.username && msg_body.starts_with(&self.bot.command_prefix.to_string())) {
                return
            }

            // Remove the prefix and split args
            let cmd = &msg_body[self.bot.command_prefix.len()..].split(" ").collect::<Vec<&str>>();

            println!("Command from `{}` in room `{}` with contents: {:?}", msg_sender, room.room_id(), cmd);

            if cmd[0] == "ping" {
                let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain(
                    "Pong 🏓",
                ));

                room.send(content, None).await.unwrap();
            }
        }
    }
}

#[tokio::main]
async fn main() {
    /*

    Basic init

    */

    // Get info from environment variables
    let homeserver_url = match std::env::var("MSB_HS_URL") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Could not get the URL of the homeserver - please set the MSB_HS_URL environment variable");
            std::process::exit(1);
        }
    };
    let username = match std::env::var("MSB_USER") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Could not get the username to log in as - please set the MSB_USER environment variable");
            std::process::exit(1);
        }
    };
    let password = match std::env::var("MSB_PASS") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Could not get the password to log in with - please set the MSB_PASS environment variable");
            std::process::exit(1);
        }
    };
    let command_prefix = match std::env::var("MSB_PREFIX") {
        Ok(value) => value,
        Err(_) => "!self ".to_string()
    };

    /*

    Set up bot struct

    */

    // Create an instance of the bot
    let bot = MxSelfBot::new(username, password, homeserver_url, command_prefix);

    // Make sure that the bot was created successfuly (homeserver URL was valid)
    let bot = match bot {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("Failed to parse homeserver URL with error: {}", msg);
            std::process::exit(1)
        },
    };

    /*

    Set up clean exit

    */

    // Keep track of whether an exit signal was received
    let is_exiting = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Register all exit signals so they can be handled
    for signal in signal_hook::consts::TERM_SIGNALS {
        // Exit with code 1 when terminated with second signal
        // First time this will do nothing as is_exiting is false
        let followup_sig_reg_result = signal_hook::flag::register_conditional_shutdown(*signal, 1, std::sync::Arc::clone(&is_exiting));
        match followup_sig_reg_result {
            Ok(_) => {},
            Err(msg) => eprintln!("Failed to register follow-up signal handler for signal `{}` due to error: {}", signal, msg),
        }
        // When the first signal is received, prepare the above by setting is_exiting to true
        // This also triggers a cleanup process
        let initial_sig_reg_result = signal_hook::flag::register(*signal, std::sync::Arc::clone(&is_exiting));
        match initial_sig_reg_result {
            Ok(_) => {},
            Err(msg) => eprintln!("Failed to register initial signal handler for signal `{}` due to error: {}", signal, msg),
        }
    }

    /*

    Run the bot

    */

    // Authenticate with the homeserver
    let login_result = bot.login().await;

    // Ensure authentication was successful
    match login_result {
        Ok(info) => println!("Authenticated to Matrix server `{}` as `{}` and got device ID `{}`", bot.homeserver_url, bot.username, info.device_id),
        Err(msg) => {
            eprintln!("Failed to authenticate to Matrix server `{}` as `{}` due to error: {}", bot.homeserver_url, bot.username, msg);
            std::process::exit(1)
        },
    }

    // Run the bot's sync loop and handle events
    println!("Starting sync loop and listening for commands via Matrix");
    let run_result = bot.run(std::sync::Arc::clone(&is_exiting)).await;

    // Detect errors in sync loop after it exits
    match run_result {
        Ok(_) => println!("Sync loop exited"),
        Err(msg) => eprintln!("Sync loop crashed due to error: {}", msg),
    }

    /*

    Clean up when bot exits

    */

    println!("Cleaning up");

    // Try logging out to remove unneeded session
    let logout_result = bot.logout().await;

    // Check if logout was successful
    match logout_result {
        Ok(()) => println!("Logged out"),
        Err(msg) => println!("Logging out failed with error: {}", msg),
    }

}