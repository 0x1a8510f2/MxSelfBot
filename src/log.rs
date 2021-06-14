#[derive(Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}
impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_uppercase())
    }
}

#[derive(Clone, Copy)]
pub struct Logger {}
impl Logger {
    pub fn new() -> Self {
        Self {}
    }

    pub fn log(&self, level: LogLevel, message: String) {
        let time = chrono::Utc::now();
        let string = format!("<{}> [{}] - {}", time.to_string(), level, message);

        match level {
            LogLevel::Error => eprintln!("{}", string),
            _ => println!("{}", string),
        }
    }
}