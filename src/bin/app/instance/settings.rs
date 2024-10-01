use log::error;

use super::PomeloError;

// List of instances to use for Invidious.
// Only instances from the official Invidious docs are used.
pub (crate) const INVID_INSTANCES: &[(&str, &str)] = &[
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

// Settings that can be changed, directly or indirectly, by the user. These settings are persistant between runs.
#[derive(serde::Serialize, serde::Deserialize)]
pub (crate) struct PomeloSettings {
    window_size: (f32, f32),
    invidious_index: usize,
    yt_dlp_use_nightly: bool,
    yt_dlp_download_folder: String,
    video_skip_on_error: bool,
}

impl PomeloSettings {
    // Create with default settings.
    pub (crate) fn new() -> Self {
        match Self::load() {
            Ok(settings) => settings,
            Err(_) => Self {
                window_size: (500.0, 500.0),
                invidious_index: 0,
                yt_dlp_use_nightly: false,
                yt_dlp_download_folder: String::from("./downloads"),
                video_skip_on_error: false
            }   
        }
    }

    pub (crate) fn window_size(&self) -> (f32, f32) {
        self.window_size
    }

    pub (crate) fn set_window_size(&mut self, width: f32, height: f32) {
        self.window_size = (width, height);
    }

    pub (crate) fn invidious_index(&self) -> usize {
        self.invidious_index
    }

    pub (crate) fn invidious_url(&self) -> &str {
        INVID_INSTANCES[self.invidious_index].0
    }

    pub (crate) fn invidious_country(&self) -> &str {
        INVID_INSTANCES[self.invidious_index].1
    }

    pub (crate) fn set_invidious_index(&mut self, index: usize) {
        self.invidious_index = index;
    }

    pub (crate) fn use_nightly(&self) -> bool {
        self.yt_dlp_use_nightly
    }

    pub (crate) fn set_use_nightly(&mut self, nightly: bool) {
        self.yt_dlp_use_nightly = nightly;
    }

    pub (crate) fn download_folder(&self) -> &str {
        &self.yt_dlp_download_folder
    }

    pub (crate) fn set_download_folder(&mut self, path: &str) {
        self.yt_dlp_download_folder = String::from(path);
    }

    pub (crate) fn video_skip_on_error(&self) -> bool {
        self.video_skip_on_error
    }

    pub (crate) fn set_video_skip_on_error(&mut self, skip: bool) {
        self.video_skip_on_error = skip;
    }

    // Load settings from the settings.json file, if it exists.
    pub (crate) fn load() -> Result<Self, PomeloError> {
        use std::io::Read;

        match std::fs::File::open("settings.json") {
            Ok(mut file) => {
                let mut buffer = String::new();
                match file.read_to_string(&mut buffer) {
                    Ok(_) => serde_json::from_str::<PomeloSettings>(buffer.as_str()).map_err(PomeloError::new),
                    Err(e) => Err(PomeloError::new(e))
                }
            },
            Err(e) => Err(PomeloError::new(e))
        }
    }

    // Serialize settings to JSON and write to file.
    pub (crate) fn save(&self) {
        use std::io::Write;

        match std::fs::File::create("settings.json") {
            Ok(mut file) => {
                match serde_json::to_string_pretty(self) {
                    Ok(pretty_json) => if let Err(e) = file.write_all(pretty_json.as_bytes()) {
                        error!("Failed to save settings: {}", e);
                    },
                    Err(e) => error!("Failed to save settings: {}", e)
                }
            },
            Err(e) => error!("Failed to save settings: {}", e)
        }
    }
}