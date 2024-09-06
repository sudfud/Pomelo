/*
 * Pomelo Version 0.2.0
 */

mod app;
mod utils;
mod yt_fetch;

//mod iced_video_player;

// List of instances to use for Invidious.
// Only instances from the official Invidious docs are used.
const INVID_INSTANCES: &[(&str, &str)] = &[
    ("https://invidious.darkness.services", "USA"),
    ("https://invidious.incogniweb.net", "USA"),
    ("https://inv.in.projectsegfau.lt", "India"),
    ("https://invidious.materialio.us", "New Zealand"),
    ("https://invidious.reallyaweso.me", "Germany"),
    ("https://invidious.privacyredirect.com", "Finland"),
    ("https://invidious.jing.rocks", "Japan"),
    ("https://inv.us.projectsegfau.lt", "USA"),
    ("https://invidious.drgns.space", "USA"),
    ("https://invidious.fdn.fr", "France"),
    ("https://iv.datura.network", "Finland"),
    ("https://yt.drgnz.club", "Czech Republic"),
    ("https://invidious.private.coffee", "Austria"),
    ("https://invidious.protokolla.fi", "Finland"),
    ("https://inv.tux.pizza", "USA"),
    ("https://inv.nadeko.net", "Chile"),
    ("https://iv.melmac.space", "Germany"),
    ("https://invidious.privacydev.net", "France"),
    ("https://invidious.flokinet.to", "Romania"),
    ("https://yt.artemislena.eu", "Germany"),
    ("https://yewtu.be", "Germany")
];

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
