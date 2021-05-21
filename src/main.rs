// MxSelfBot by 0x1a8510f2

// Include modules
mod cmds;

// Bring some stuff into the scope
use url;
use matrix_sdk::{
    self, async_trait,
    events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent}, SyncMessageEvent,
    },
    room::Room,
    Client, ClientConfig, EventHandler, SyncSettings,
};
use locales::t;

// Keep track of the available languages to avoid panics
static AVAIL_LANG: [&'static str; 1] = [
    "en_GB",
];

// The bot struct
#[derive(Clone)]
pub struct MxSelfBot {
    version: String,
    description: String,
    source: String,
    client: Client,
    username: String,
    password: String,
    homeserver_url: String,
    command_prefix: String,
    lang: String,
}
impl MxSelfBot {
    // Create an instance of the bot with the given credentials
    // Checks the provided homeserver_url and only creates the instance if the URL is valid
    pub fn new(username: String, password: String, homeserver_url: String, command_prefix: String, lang: String) -> Result<Self, url::ParseError> {

        // Create a client
        let client_config = ClientConfig::new();
        let homeserver_url_parsed = url::Url::parse(&homeserver_url)?;
        let client = Client::new_with_config(homeserver_url_parsed, client_config).unwrap();

        Ok(Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: env!("CARGO_PKG_DESCRIPTION").to_string(),
            source: env!("CARGO_PKG_REPOSITORY").to_string(),
            client,
            username,
            password,
            homeserver_url,
            command_prefix,
            lang,
        })
    }

    // Authenticate with the homeserver using details provided when creating the bot instance
    pub async fn login(&self) -> Result<matrix_sdk::api::r0::session::login::Response, matrix_sdk::Error> {

        let login_result = self.client
            .login(&self.username, &self.password, None, Some(&t!("app_name", self.lang)))
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
        Err(t!("err.auth.logout_tmp_unsupported", device_id: &device_id, self.lang))

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
            let cmdstring = cmd.join(" ");

            println!("{}", t!("info.command.recv", cmdline: &cmdstring, room_id: room.room_id().as_str(), sender: msg_sender.as_str(), self.bot.lang));

            let result = cmds::execute(cmd, event, &room, &self.bot).await;
            match result {
                Some(result) => {
                    // If there is a result to be sent, send it
                    let send_result = room.send(result, None).await;
                    match send_result {
                        Ok(_) => {},
                        Err(msg) => eprintln!("{}", t!("err.matrix.event_send", err: &msg.to_string(), self.bot.lang)),
                    }
                },
                None => {},
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
    let lang = match std::env::var("MSB_LANG") {
        Ok(value) => {
            // If the specified language code is invalid, fall back to the first available
            // and issue a warning
            if AVAIL_LANG.contains(&&*value) { value }
            else {
                let lang = AVAIL_LANG[0];
                eprintln!("{}", t!("err.conf.unavailable_lang_code", fallback_code: lang, req_code: &value, lang));
                lang.to_string()
            }
        },
        Err(_) => AVAIL_LANG[0].to_string(),
    };
    let homeserver_url = match std::env::var("MSB_HS_URL") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("{}", t!("err.conf.unset_required.homeserver", lang));
            std::process::exit(1);
        }
    };
    let username = match std::env::var("MSB_USER") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("{}", t!("err.conf.unset_required.user", lang));
            std::process::exit(1);
        }
    };
    let password = match std::env::var("MSB_PASS") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("{}", t!("err.conf.unset_required.pass", lang));
            std::process::exit(1);
        }
    };
    let command_prefix = match std::env::var("MSB_PREFIX") {
        Ok(value) => value,
        Err(_) => {
            let default = "!s ";
            println!("{}", t!("warn.conf.unset.prefix", default: default, lang));
            default.to_string()
        }
    };

    /*

    Set up bot struct

    */

    // Create an instance of the bot
    let bot = MxSelfBot::new(username, password, homeserver_url, command_prefix, lang.clone());

    // Make sure that the bot was created successfuly (homeserver URL was valid)
    let bot = match bot {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("{}", t!("err.conf.invalid.homeserver", err: &msg.to_string(), lang));
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
            Err(msg) => eprintln!("{}", t!("err.signal.register.followup", err: &msg.to_string(), signal: &signal.to_string(), lang)),
        }
        // When the first signal is received, prepare the above by setting is_exiting to true
        // This also triggers a cleanup process
        let initial_sig_reg_result = signal_hook::flag::register(*signal, std::sync::Arc::clone(&is_exiting));
        match initial_sig_reg_result {
            Ok(_) => {},
            Err(msg) => eprintln!("{}", t!("err.signal.register.initial", err: &msg.to_string(), signal: &signal.to_string(), lang)),
        }
    }

    /*

    Run the bot

    */

    // Authenticate with the homeserver
    let login_result = bot.login().await;

    // Ensure authentication was successful
    match login_result {
        Ok(info) => println!("{}", t!("info.auth.login_success", device_id: info.device_id.as_str(), hs_url: &bot.homeserver_url, username: &bot.username, lang)),
        Err(msg) => {
            eprintln!("{}", t!("err.auth.login_fail", err: &msg.to_string(), hs_url: &bot.homeserver_url, username: &bot.username, lang));
            std::process::exit(1)
        },
    }

    // Run the bot's sync loop and handle events
    println!("{}", t!("info.sync.start", lang));
    let run_result = bot.run(std::sync::Arc::clone(&is_exiting)).await;

    // Detect errors in sync loop after it exits
    match run_result {
        Ok(_) => println!("{}", t!("info.sync.stop", lang)),
        Err(msg) => eprintln!("{}", t!("err.sync.crash", err: &msg.to_string(), lang)),
    }

    /*

    Clean up when bot exits

    */

    println!("{}", t!("info.app.cleanup_start", lang));

    // Try logging out to remove unneeded session
    let logout_result = bot.logout().await;

    // Check if logout was successful
    match logout_result {
        Ok(()) => println!("{}", t!("info.auth.logout_success", lang)),
        Err(msg) => eprintln!("{}", t!("err.auth.logout_fail", err: &msg.to_string(), lang)),
    }

}