use std::io::BufRead;
use std::path::Path;

use iced::{Task, Length};
use iced::widget::{column, Column, Text};

use invidious::universal::Playlist;

use log::{info, error};

use crate::app::instance::cache::PomeloCache;
use crate::app::PomeloError;
use crate::INVID_INSTANCES;

use super::{PomeloInstance, DownloadInfo, Msg, Navigation};
use super::VideoOrder;

#[derive(Debug, Clone)]
pub (crate) enum PlaylistInfoMessage {
    LoadPlaylist(String),
    LoadComplete(Result<Playlist, PomeloError>),
    ToVideo(VideoOrder),
    StartDownload,
    NextChunk(String, Result<usize, PomeloError>),
    DownloadCancelled,
    DownloadComplete(Result<(), PomeloError>)
}

impl From<PlaylistInfoMessage> for Msg {
    fn from(value: PlaylistInfoMessage) -> Self {
        Msg::PlaylistInfo(value)
    }
}

// Displays a list of videos from given playlist, with options for playback and downloading.
#[derive(Default)]
pub (crate) struct PlaylistInfoPage {
    playlist: Option<Playlist>,
    videos: Vec<String>,
    downloading: bool,
    download_info: Option<DownloadInfo>,
    download_index: usize,
    error: Option<PomeloError>
}

impl super::PomeloPage for PlaylistInfoPage {
    fn update(&mut self, instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation) {

        // Return to search results page.
        if let Msg::Back = message {
            return (Task::none(), Navigation::Back);
        }

        else if let Msg::PlaylistInfo(msg) = message {
            match msg {
                PlaylistInfoMessage::LoadPlaylist(id) 
                    => return self.load_playlist(id, instance.settings().invidious_index()),

                PlaylistInfoMessage::LoadComplete(result)
                    => return self.on_load_complete(result, instance.cache()),

                PlaylistInfoMessage::ToVideo(order)
                    => return self.go_to_video(order),

                PlaylistInfoMessage::StartDownload
                    => return self.start_download(instance),

                PlaylistInfoMessage::NextChunk(output, result)
                    => return self.on_next_chunk(output, result),

                PlaylistInfoMessage::DownloadCancelled
                    => return on_download_cancelled(instance),

                PlaylistInfoMessage::DownloadComplete(result)
                    => self.on_download_complete(result)
            }
        }

        (Task::none(), Navigation::None)
    }

    fn view(&self, instance: &PomeloInstance) -> iced::Element<Msg> {
        use iced::widget::{row, ProgressBar};
        use super::{centered_text_button, ConditionalMessage, FillElement};
        
        let mut column = Column::new().spacing(10).align_x(iced::Alignment::Center);

        match &self.playlist {
            Some(playlist) => {

                column = column.push(self.create_playlist_element(playlist, instance));
                    

                if let Some(e) = &self.error {
                    column = column.push(Text::new(&e.error));
                }

                // Draw download progress bars and cancel button
                if self.downloading {

                    let info = self.download_info.as_ref().unwrap();

                    column = column.extend(
                        vec![     
                            ProgressBar::new(
                                0.0..=playlist.video_count as f32,
                                self.download_index as f32
                            ).width(instance.settings().window_size().0 / 2.0).into(),

                            ProgressBar::new(
                                0.0..=info.length as f32,
                                info.progress as f32
                            ).width(instance.settings().window_size().0 / 2.0).into(),

                            centered_text_button("Cancel", Some(200), None::<Length>)
                                .on_press(PlaylistInfoMessage::DownloadCancelled.into())
                                .into()
                        ]
                    );
                }

                // Draw playback and download buttons.
                else {      
                    column = column.extend(
                        vec![
                            row![
                                centered_text_button("Shuffle", Some(100), None::<Length>)
                                    .on_press(
                                        PlaylistInfoMessage::ToVideo(VideoOrder::Shuffled).into()
                                    ),

                                centered_text_button("Reverse", Some(100), None::<Length>)
                                    .on_press(
                                        PlaylistInfoMessage::ToVideo(VideoOrder::Reversed).into()
                                    )
                            ].spacing(10).into(),

                            centered_text_button("Download Playlist", Some(200), None::<Length>)
                                .on_press(PlaylistInfoMessage::StartDownload.into())
                                .into()
                        ]
                    );
                }
            },
            None => column = column.push("Loading...")
        }

        column = column.push(
            centered_text_button("Back", Some(100), None::<Length>)
                .on_press_maybe(
                    Msg::Back.on_condition(
                        !self.downloading && (self.playlist.is_some() || self.error.is_some())
                    )
                )
        );

        column.fill()
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<Msg> {
        iced::Subscription::none()
    }
}

impl PlaylistInfoPage {
    pub (crate) fn new() -> Self {
        Default::default()
    }

    // Get info for the playlist with the given id from Indivious
    fn load_playlist(&self, id: String, instance_index: usize) -> (Task<Msg>, Navigation) {
        use crate::yt_fetch::VideoFetcher;

        info!("Loading playlist info from id: {}", id);

        let downloader = VideoFetcher::new(String::from(INVID_INSTANCES[instance_index].0));
        (
            Task::perform(
                async move {
                    downloader.get_playlist_videos(&id).await.map_err(PomeloError::new)
                },
                |result| PlaylistInfoMessage::LoadComplete(result).into()
            ),
            Navigation::None
        )
    }

    // Handles the result from loading playlist info. Starts loading thumbnails if it was successful.
    fn on_load_complete(&mut self, result: Result<Playlist, PomeloError>, cache: &PomeloCache) -> (Task<Msg>, Navigation) {
        use crate::yt_fetch::SearchResults;

        let command = match result {
            Ok(playlist) => {
                self.playlist = Some(playlist.clone());
                self.videos = playlist.videos.iter()
                    .map(|v| v.id.clone())
                    .collect();
                super::batch_thumbnail_commands(&SearchResults::PlaylistVideos(playlist.clone()), cache)
            },
            Err(e) => {
                error!("Failed to load playlist info: {}", e.error);
                self.error = Some(e);
                Task::none()
            }
        };

        (command, Navigation::None)
    }

    // Move to the video player, play videos in given order.
    fn go_to_video(&self, order: VideoOrder) -> (Task<Msg>, Navigation) {
        use super::video_player_page::{VideoPlayerPage, VideoPlayerMessage};

        let videos = self.videos.iter().cloned()
            .map(|v| (v, false))
            .collect();

        let index = if let VideoOrder::Sequential(i) = order {i} else {0};

        (
            Task::done(VideoPlayerMessage::LoadVideo(index).into()),
            Navigation::GoTo(Box::new(VideoPlayerPage::new(videos, order)))
        )
    }

    // Setup yt-dlp process for downmloading the playlist.
    fn start_download(&mut self, instance: &mut PomeloInstance) -> (Task<Msg>, Navigation) {
        use filenamify::filenamify;

        let playlist = self.playlist.as_ref().unwrap();
        let channel = filenamify(&playlist.author);
        let title = filenamify(&playlist.title);
        let out_path = format!("./downloads/playlists/{} - {}", channel, title);

        let args = [
            &playlist.id,
            "-P",
            &out_path,
            "-q",
            "--no-warnings",
            "--progress",
            "--newline",
            "--progress-template",
            "download:%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(info.playlist_index)s",
            "--output",
            "%(playlist_index)s - %(title)s [%(id)s].%(ext)s"
        ];

        if !Path::exists(Path::new(&out_path)) {
            let _ = std::fs::create_dir(&out_path);
        }

        let command = match instance.create_download_process(&args) {
            Ok((mut stdout, stderr)) => {
                let mut output = String::new();
                let result = stdout.read_line(&mut output);

                self.downloading = true;
                self.download_info = Some(DownloadInfo::new(out_path, stdout, stderr));

                Task::done(PlaylistInfoMessage::NextChunk(output, result.map_err(PomeloError::new)).into())
            },

            Err(e) => Task::done(PlaylistInfoMessage::DownloadComplete(Err(e)).into())
        };

        (command, Navigation::None)
    }

    // Called when yt-dlp collects a chunk of bytes. Info from yt-dlp is used to update UI during download.
    fn on_next_chunk(&mut self, output: String, result: Result<usize, PomeloError>) -> (Task<Msg>, Navigation) {
        let command = match result {
            Ok(index) => match index {
                0 => Task::done(PlaylistInfoMessage::DownloadComplete(Ok(())).into()),
                _ => {

                    let info = self.download_info.as_mut().unwrap();

                    // Read formatted progress string from yt-dlp
                    let nums: Vec<&str> = output.trim().split('|').collect();

                    if let Some(n_str) = nums.first() {
                        if let Ok(n) = n_str.parse() {
                            info.progress = n;
                        }
                    }

                    if let Some(n_str) = nums.get(1) {
                        if let Ok(n) = n_str.parse() {
                            info.length = n;
                        }
                    }

                    if let Some(n_str) = nums.get(2) {
                        if let Ok(n) = n_str.parse() {
                            self.download_index = n;
                        }
                    }

                    let mut output = String::new();
                    let result = info.stdout
                        .read_line(&mut output)
                        .map_err(PomeloError::new);

                    Task::done(PlaylistInfoMessage::NextChunk(output, result).into())
                }
            },

            Err(e) => Task::done(PlaylistInfoMessage::DownloadComplete(Err(e)).into())
        };

        (command, Navigation::None)
    }

    // Download has finished, or the download was stopped by an error or by the user.
    fn on_download_complete(&mut self, result: Result<(), PomeloError>) {
        self.downloading = false;

        if let Err(e) = result {
            self.error = Some(e);
        }

        else {
            let info = self.download_info.take().unwrap();

            if let Some(Ok(line)) = info.stderr.lines().last() {
                error!("Download failed: {}", line);
                self.error = Some(PomeloError::from(line));
            }

            else {
                info!("Video downloaded to file: {:?}", Path::new(&info.path));
            }
        }
    }

    // Generates a scrollable list of playlist videos.
    fn create_playlist_element(&self, playlist: &Playlist, instance: &PomeloInstance) -> iced::Element<Msg> {
        use iced::widget::{Row, Button, Scrollable, Image};
    
        let mut vids = Column::<Msg>::new().spacing(10);
        for (i, video) in playlist.videos.iter().enumerate() {
            let mut row: Row<Msg> = Row::new();
    
            if let Some(handle) = instance.cache().get_thumbnail(&video.id) {
                row = row.push(Image::new(handle.clone()));
            }
    
            row = row.push(
                column![
                    Text::new(format!("{}. {}", i+1, video.title.clone())),
                    Text::new(video.author.clone())
                ]
            );
    
            vids = vids.push(
                Button::new(row)
                    .width(Length::Fill)
                    .on_press(PlaylistInfoMessage::ToVideo(VideoOrder::Sequential(i)).into())
            );        
        }
    
        Scrollable::new(vids)
            .width(Length::Fill)
            .height(instance.settings().window_size().1 * 3.0 / 4.0)
            .into()
    }
}

// The download was cancelled by the user.
fn on_download_cancelled(instance: &mut PomeloInstance) -> (Task<Msg>, Navigation) {
    instance.cancel_download();

    (
        Task::done(PlaylistInfoMessage::DownloadComplete(Err(PomeloError::from("Cancelled by user."))).into()),
        Navigation::None
    )
}