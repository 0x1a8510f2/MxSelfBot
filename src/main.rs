/*

MxSelfBot by 0x1a8510f2

*/

// Include modules
mod log;
mod cmds;
mod context;
mod mxclient;

// Bring some stuff into the scope
use locales::t;
use log::{Logger, LogLevel};

// Keep track of the available languages to avoid panics
static AVAIL_LANG: [&'static str; 1] = [
    "en_GB",
];

// Some metadata about the project
lazy_static::lazy_static! {
    static ref META: std::collections::HashMap<&'static str, &'static str> = {
        let mut m = std::collections::HashMap::new();
        m.insert("VERSION", env!("CARGO_PKG_VERSION"));
        m.insert("VERSION_MAJOR", env!("CARGO_PKG_VERSION_MAJOR"));
        m.insert("VERSION_MINOR", env!("CARGO_PKG_VERSION_MINOR"));
        m.insert("VERSION_PATCH", env!("CARGO_PKG_VERSION_PATCH"));
        m.insert("AUTHORS", env!("CARGO_PKG_AUTHORS"));
        m.insert("DESCRIPTION", env!("CARGO_PKG_DESCRIPTION"));
        m.insert("REPOSITORY", env!("CARGO_PKG_REPOSITORY"));
        m
    };
}

// The important stuff
#[tokio::main]
async fn main() {
    /*

    Basic init

    */

    // Set up logging
    let logger = Logger::new();

    // Get info/config from environment variables
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
    // Use the app name as the session name
    let connection_result = mxclient::connect(&homeserver_url, &username, &password, &t!("app_name", lang)).await;

    // Make sure session creation was successful
    let client = match connection_result {
        Ok(c) => c,
        Err(msg) => {
            logger.log(LogLevel::Error, t!("err.auth.connect_fail", err: &msg.to_string(), hs_url: &homeserver_url, username: &username, lang));
            std::process::exit(1); // TODO: Handle certain errors like timeouts gracefully
        },
    };

    /*

    Set up clean exit in case of received signal

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
    let run_result = mxclient::run(&client, Box::new(mxclient::MxSelfBotEventHandler::new(
        META.clone(),
        username.clone(),
        command_prefix.clone(),
        lang.clone(),
        logger,
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
    let _disconnect_result = mxclient::disconnect(&client).await;

    // Check if logout was successful
    /*match disconnect_result {
        Ok(()) => logger.log(LogLevel::Info, t!("info.auth.logout_success", lang)),
        Err(msg) => logger.log(LogLevel::Error, t!("err.auth.disconnect_fail", err: &msg.to_string(), lang)),
    }*/

    let device_id = client.device_id().await.unwrap();
    logger.log(LogLevel::Warn, t!("warn.auth.disconnect_tmp_unsupported", device_id: &device_id.as_str(), lang))

}