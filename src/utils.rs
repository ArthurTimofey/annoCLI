use colored::*;

pub enum LoggerSeverity {
    Info,
    Warning,
    Error,
}

pub fn logger(severity: LoggerSeverity, message: &str) {
    let mut log_message = String::new();

    let color = match severity {
        LoggerSeverity::Info => Color::Green,
        LoggerSeverity::Warning => Color::Yellow,
        LoggerSeverity::Error => Color::Red,
    };

    let prefix = match severity {
        LoggerSeverity::Info => "[INFO]",
        LoggerSeverity::Warning => "[WARNING]",
        LoggerSeverity::Error => "[ERROR]",
    };

    log_message.push_str(prefix);
    log_message.push_str(" ");
    log_message.push_str(message);

    println!("{}", log_message.color(color));
}
