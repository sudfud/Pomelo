use log::error;

use super::PomeloError;

// Settings that can be changed, directly or indirectly, by the user. These settings are persistant between runs.
#[derive(serde::Serialize, serde::Deserialize)]
pub (crate) struct PomeloSettings {
    window_size: (f32, f32),
    invidious_index: usize,
    yt_dlp_use_nightly: bool
}

impl PomeloSettings {
    // Create with default settings.
    pub (crate) fn new() -> Self {
        Self {
            window_size: (500.0, 500.0),
            invidious_index: 0,
            yt_dlp_use_nightly: false
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

    pub (crate) fn set_invidious_index(&mut self, index: usize) {
        self.invidious_index = index;
    }

    pub (crate) fn use_nightly(&self) -> bool {
        self.yt_dlp_use_nightly
    }

    pub (crate) fn set_use_nightly(&mut self, nightly: bool) {
        self.yt_dlp_use_nightly = nightly;
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