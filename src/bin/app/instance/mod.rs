pub (crate) mod settings;
pub (crate) mod cache;

use log::{info, warn, error};

use super::PomeloError;

use self::settings::PomeloSettings;
use self::cache::PomeloCache;



// Readers for the yt-dlp process' stdout and stderr
type DownloadReader = (
    std::io::BufReader<std::process::ChildStdout>,
    std::io::BufReader<std::process::ChildStderr>
);

// Collection of items that'll be used during the program's runtime.
pub (crate) struct PomeloInstance {
    settings: PomeloSettings,
    cache: PomeloCache,
    download_process: Option<std::process::Child>
}

impl PomeloInstance {
    pub (crate) fn new(settings: PomeloSettings) -> Self {
        Self {
            settings,
            cache: PomeloCache::new(),
            download_process: None
        }
    }

    // Mutable and immutable getters
    pub (crate) fn settings(&self) -> &PomeloSettings {
        &self.settings
    }

    pub (crate) fn settings_mut(&mut self) -> &mut PomeloSettings {
        &mut self.settings
    }

    pub (crate) fn cache(&self) -> &PomeloCache {
        &self.cache
    }

    pub (crate) fn cache_mut(&mut self) -> &mut PomeloCache {
        &mut self.cache
    }

    // Build and run a command for yt-dlp, returns a reader for stdout and stderr if successful.
    pub (crate) fn create_download_process(&mut self, args: &[&str]) -> Result<DownloadReader, PomeloError> {
        use std::process::{Command, Stdio};

        match self.yt_dlp_check() {
            Ok(yt_dlp_path) => {
                let mut command = &mut Command::new(yt_dlp_path);
    
                command = command
                    .args(args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
    
                command.spawn()
                .map(|mut child| {
                    let stdout = child.stdout
                        .take()
                        .map(std::io::BufReader::new)
                        .unwrap();
    
                    let stderr = child.stderr
                        .take()
                        .map(std::io::BufReader::new)
                        .unwrap();
    
                    self.download_process = Some(child);
    
                    (stdout, stderr)
                })
                .map_err(PomeloError::new)
            },
    
            Err(e) => Err(e)
        }
           
    }

    // Kill the yt-dlp process.
    pub (crate) fn cancel_download(&mut self) {
        if let Some(mut child) = self.download_process.take() {
            match child.kill() {
                Ok(_) => info!("Download cancelled. Yt-dlp process successfully killed."),
                Err(e) => error!("Failed to kill yt-dlp process: {}", e)
            }
        }
    }
    
    // Checks if yt-dlp exists. If it does, try to update it. If not, download it.
    fn yt_dlp_check(&self) -> Result<String, PomeloError> {
        use std::path::Path;

        let path_str = String::from("./yt-dlp");
    
        if !Path::exists(Path::new(&path_str)) {
            let _ = std::fs::create_dir(&path_str);
        }     
    
        let filename = if cfg!(target_os = "windows") {
            "/yt-dlp.exe"
        } else {
            "/yt-dlp"
        };
    
        let yt_dlp_path = [&path_str, filename].concat();
    
        if !Path::exists(Path::new(&yt_dlp_path)) {
            // Download yt-dlp
            info!("Yt-dlp not found. Downloading...");
            if let Err(e) = futures::executor::block_on(youtube_dl::download_yt_dlp(&path_str)) {
                error!("Failed to download yt-dlp: {}", e);
                Err(PomeloError::new(e))
            }
            else {
                info!("Yt-dlp download complete.");
                Ok(yt_dlp_path)
            }
        }
        else {
            self.update_yt_dlp(&yt_dlp_path);
            Ok(yt_dlp_path)
        }
    }

    // Update yt-dlp to latest stable or nightly release.
    fn update_yt_dlp(&self, yt_dlp_path: &str) {
        use std::process::Command;

        info!("Checking for yt-dlp update...");

        let mut cmd = &mut Command::new(yt_dlp_path);
        cmd = cmd.args(
            [
                "--update-to",
                if self.settings.use_nightly() {
                    "nightly@latest"
                } else {
                    "stable@latest"
                }
            ]
        );

        if let Err(e) = cmd.output() {
            warn!("Failed to update yt-dlp: {}", e);
        }
        else {
            info!("Yt-dlp up to date.");
        }
    }
}