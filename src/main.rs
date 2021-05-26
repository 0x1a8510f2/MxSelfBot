// MxSelfBot by 0x1a8510f2

// Include modules
mod log;
mod cmds;
mod context;

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
use log::{Logger, LogLevel};

// Keep track of the available languages to avoid panics
static AVAIL_LANG: [&'static str; 1] = [
    "en_GB",
];

// Authenticate with the homeserver and return an instance of matrix_sdk::Client
async fn connect(hs_url: &str, username: &str, password: &str, device_name: &str)
    -> Result<matrix_sdk::Client, Box<dyn std::error::Error>> {

    let client_config = ClientConfig::new();
    let hs_url_parsed = url::Url::parse(&hs_url)?;
    let client = Client::new_with_config(hs_url_parsed, client_config)?;

    client.login(&username, &password, None, Some(&device_name)).await?;

    Ok(client)
}

// End session with homeserver
async fn disconnect(_client: &matrix_sdk::Client) -> Result<(), String> {
    // TODO
    // Waiting on https://github.com/matrix-org/matrix-rust-sdk/issues/115

    /*if self.client.logged_in().await {
    }*/
    Ok(())
}

// Run the sync loop and handle events until kill_loop is set to false
async fn run(client: &matrix_sdk::Client, eh: Box<MxSelfBotEventHandler>, kill_loop: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Result<(), matrix_sdk::Error> {
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
struct MxSelfBotEventHandler { ctx: context::Ctx }
impl MxSelfBotEventHandler { fn new(
    username: String,
    command_prefix: String,
    lang: String,
) -> Self {
    let ctx = context::Ctx::new(
        username,
        command_prefix,
        Vec::new(),
        lang,
    );
    Self { ctx }
} }

// The logic behind MxSelfBotEventHandler
#[async_trait]
impl EventHandler for MxSelfBotEventHandler {
    // Handle message events (usually commands)
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
            if !(*msg_sender == self.ctx.username && msg_body.starts_with(&self.ctx.command_prefix)) {
                return
            }

            // Create a copy of the context with command-specific options filled in
            let mut tmp_ctx = self.ctx.clone();
            tmp_ctx.cmdline = msg_body[self.ctx.command_prefix.len()..].split(" ").collect::<Vec<&str>>().iter().map(|s| String::from(*s)).collect();

            println!("{}", t!("info.command.recv", cmdline: &tmp_ctx.cmdline.join(" "), room_id: room.room_id().as_str(), sender: msg_sender.as_str(), self.ctx.lang));

            // Pass the newly-created context to the command executor so it can execute the command
            let result = cmds::execute(tmp_ctx).await;

            // Handle the results of the command execution
            match result {
                Some(result) => {
                    // If there is a result to be sent, send it
                    let send_result = room.send(result, None).await;
                    match send_result {
                        Ok(_) => {},
                        Err(msg) => eprintln!("{}", t!("err.matrix.event_send", err: &msg.to_string(), self.ctx.lang)),
                    }
                },
                None => {},
            }
        }
    }
}

// Some metadata about the project
lazy_static::lazy_static! {
    static ref VERSION: String = env!("CARGO_PKG_VERSION").to_string();
    static ref VERSION_MAJOR: String = env!("CARGO_PKG_VERSION_MAJOR").to_string();
    static ref VERSION_MINOR: String = env!("CARGO_PKG_VERSION_MINOR").to_string();
    static ref VERSION_PATCH: String = env!("CARGO_PKG_VERSION_PATCH").to_string();
    static ref AUTHORS: String = env!("CARGO_PKG_AUTHORS").to_string();
    static ref DESCRIPTION: String = env!("CARGO_PKG_DESCRIPTION").to_string();
    static ref REPOSITORY: String = env!("CARGO_PKG_REPOSITORY").to_string();
}

#[tokio::main]
async fn main() {
    /*

    Basic init

    */

    // Set up logging
    let logger = Logger::new();

    // Get info from environment variables
    let lang = match std::env::var("MSB_LANG") {
        Ok(value) => {
            // If the specified language code is invalid, fall back to the first available
            // and issue a warning
            if AVAIL_LANG.contains(&&*value) { value }
            else {
                let lang = AVAIL_LANG[0];
                logger.log(LogLevel::Warn, t!("err.conf.unavailable_lang_code", fallback_code: lang, req_code: &value, lang));
                lang.to_string()
            }
        },
        Err(_) => AVAIL_LANG[0].to_string(),
    };
    let homeserver_url = match std::env::var("MSB_HS_URL") {
        Ok(value) => value,
        Err(_) => {
            logger.log(LogLevel::Error, t!("err.conf.unset_required.homeserver", lang));
            std::process::exit(1);
        }
    };
    let username = match std::env::var("MSB_USER") {
        Ok(value) => value,
        Err(_) => {
            logger.log(LogLevel::Error, t!("err.conf.unset_required.user", lang));
            std::process::exit(1);
        }
    };
    let password = match std::env::var("MSB_PASS") {
        Ok(value) => value,
        Err(_) => {
            logger.log(LogLevel::Error, t!("err.conf.unset_required.pass", lang));
            std::process::exit(1);
        }
    };
    let command_prefix = match std::env::var("MSB_PREFIX") {
        Ok(value) => value,
        Err(_) => {
            let default = "!s ";
            logger.log(LogLevel::Warn, t!("warn.conf.unset.prefix", default: default, lang));
            default.to_string()
        }
    };

    /*

    Set up client

    */

    // Connect to and auth with homeserver
    let connection_result = connect(&homeserver_url, &username, &password, &t!("app_name", lang)).await;

    // Make sure session creation was successful
    let client = match connection_result {
        Ok(c) => c,
        Err(msg) => {
            logger.log(LogLevel::Error, t!("err.auth.connect_fail", err: &msg.to_string(), hs_url: &homeserver_url, username: &username, lang));
            std::process::exit(1); // TODO: Handle certain errors like timeouts gracefully
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
            Err(msg) => logger.log(LogLevel::Error, t!("err.signal.register.followup", err: &msg.to_string(), signal: &signal.to_string(), lang)),
        }
        // When the first signal is received, prepare the above by setting is_exiting to true
        // This also triggers a cleanup process
        let initial_sig_reg_result = signal_hook::flag::register(*signal, std::sync::Arc::clone(&is_exiting));
        match initial_sig_reg_result {
            Ok(_) => {},
            Err(msg) => logger.log(LogLevel::Error, t!("err.signal.register.initial", err: &msg.to_string(), signal: &signal.to_string(), lang)),
        }
    }

    /*

    Run the bot

    */

    // Run the bot's sync loop and handle events
    logger.log(LogLevel::Info, t!("info.sync.start", lang));
    let run_result = run(&client, Box::new(MxSelfBotEventHandler::new(
        username.clone(),
        command_prefix.clone(),
        lang.clone(),
    )), std::sync::Arc::clone(&is_exiting)).await;

    // Detect errors in sync loop after it exits
    match run_result {
        Ok(_) => logger.log(LogLevel::Info, t!("info.sync.stop", lang)),
        Err(msg) => logger.log(LogLevel::Error, t!("err.sync.crash", err: &msg.to_string(), lang)),
    }

    /*

    Clean up when bot exits

    */

    logger.log(LogLevel::Info, t!("info.app.cleanup_start", lang));

    // Try logging out to remove unneeded session
    let _disconnect_result = disconnect(&client).await;

    // Check if logout was successful
    /*match disconnect_result {
        Ok(()) => logger.log(LogLevel::Info, t!("info.auth.logout_success", lang)),
        Err(msg) => logger.log(LogLevel::Error, t!("err.auth.disconnect_fail", err: &msg.to_string(), lang)),
    }*/
    let device_id = client.device_id().await.unwrap();
    logger.log(LogLevel::Warn, t!("warn.auth.disconnect_tmp_unsupported", device_id: &device_id.as_str(), lang))

}