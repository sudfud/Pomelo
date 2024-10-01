/*
 * Pomelo Version 0.2.0
 */

mod app;
mod utils;

fn main() {
    use std::path::Path;
    use std::time::SystemTime;
    use log::{warn, LevelFilter};
    use chrono::{DateTime, Utc};
    use app::PomeloApp;

    // Get current timestamp and use it as the log's filename.
    let datetime: DateTime<Utc> = SystemTime::now().into();
    let date_str = datetime.format("%F");
    let time_str = datetime.format("%H-%M-%S");
    let log_dir = format!("./logs/{}", date_str);
    let log_file = format!(
        "{}/log-{}-{}.txt",
        log_dir,
        date_str,
        time_str
    );

    // Check for log directory, create it if it doesn't exist.
    if !Path::exists(Path::new(&log_dir)) {
        if let Err(e) = std::fs::create_dir_all(log_dir) {
            warn!("Log directory could not be found or created: {}", e)
        }
    }

    if let Err(e) = simple_logging::log_to_file(log_file, LevelFilter::Info) {
        simple_logging::log_to_stderr(LevelFilter::Info);
        warn!("Failed to setup log file. Logging to stderr instead: {}", e);
    };

    // Run Pomelo
    match iced::daemon(PomeloApp::title, PomeloApp::update, PomeloApp::view)
        .subscription(PomeloApp::subscription)
        .run_with(PomeloApp::new)
    {
        Ok(_) => println!("Goodbye!"),
        Err(e) => eprintln!("{}", e)
    }
}
