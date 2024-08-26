use std::io::BufRead;
use std::path::Path;

use iced::{Task, Length};
use iced::widget::{column, Column, Text};

use invidious::universal::Playlist;

use log::{info, error};

use crate::app::instance::cache::PomeloCache;
use crate::app::{DownloadFormat, DownloadQuality, PomeloError};
use crate::INVID_INSTANCES;

use super::{PomeloInstance, DownloadInfo, Msg, Navigation};
use super::VideoOrder;

#[derive(Debug, Clone)]
pub (crate) enum PlaylistInfoMessage {
    LoadPlaylist(String),
    LoadComplete(Result<Playlist, PomeloError>),
    ToVideo(VideoOrder)
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
    selected_format: DownloadFormat,
    selected_quality: DownloadQuality,
    downloading: bool,
    download_info: Option<DownloadInfo>,
    download_index: usize,
    error: Option<PomeloError>
}

impl super::PomeloPage for PlaylistInfoPage {
    fn update(&mut self, instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation) {

        match message {
            Msg::Back => return (Task::none(), Navigation::Back),
            Msg::Home => return (Task::none(), Navigation::Home),
            Msg::SetDownloadFormat(format) => self.selected_format = format,
            Msg::SetDownloadQuality(quality) => self.selected_quality = quality,
            Msg::StartVideoDownload => return self.start_download(instance),
            Msg::NextVideoChunk(line, result) => return self.on_next_chunk(line, result),
            Msg::VideoDownloadCancelled => return on_download_cancelled(instance),
            Msg::VideoDownloadComplete(result) => self.on_download_complete(result),

            Msg::PlaylistInfo(msg) => match msg {
                PlaylistInfoMessage::LoadPlaylist(id) 
                    => return self.load_playlist(id, instance.settings().invidious_index()),

                PlaylistInfoMessage::LoadComplete(result)
                    => return self.on_load_complete(result, instance.cache()),

                PlaylistInfoMessage::ToVideo(order)
                    => return self.go_to_video(order),
            }

            _ => ()
        }

        (Task::none(), Navigation::None)
    }

    fn view(&self, instance: &PomeloInstance) -> iced::Element<Msg> {
        use iced::widget::{row, ProgressBar, Text, Button, Scrollable};
        use super::{download_element, ConditionalMessage, FillElement};
        
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

                            Button::new(Text::new("Cancel").center())
                                .width(200)
                                .on_press(Msg::VideoDownloadCancelled.into())
                                .into()
                        ]
                    );
                }

                // Draw playback and download buttons.
                else {      
                    column = column.push(
                        column![
                            row![
                                Button::new(Text::new("Shuffle").center())
                                    .width(100)
                                    .on_press(
                                        PlaylistInfoMessage::ToVideo(VideoOrder::Shuffled).into()
                                    ),

                                Button::new(Text::new("Reverse").center())
                                    .width(100)
                                    .on_press(
                                        PlaylistInfoMessage::ToVideo(VideoOrder::Reversed).into()
                                    )
                            ].spacing(10),

                            download_element(&self.selected_format, &self.selected_quality),

                            column![
                                Button::new(Text::new("Back").center())
                                    .width(100)
                                    .on_press_maybe(
                                        Msg::Back.on_condition(
                                            !self.downloading && (self.playlist.is_some() || self.error.is_some())
                                        )
                                    ),

                                Button::new(Text::new("Home").center())
                                    .width(100)
                                    .on_press_maybe(
                                        Msg::Home.on_condition(
                                            !self.downloading && (self.playlist.is_some() || self.error.is_some())
                                        )
                                    )
                            ].spacing(25)
                        ].spacing(50).align_x(iced::Alignment::Center)
                    );
                }
            },
            None => column = column.push("Loading...")
        }

        Scrollable::new(column.width(iced::Length::Fill)).fill()
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
        let out_path = format!("{}/playlists/{}/{} - {}",
            instance.settings().download_folder(),
            if self.selected_format.is_audio() { "audio" } else { "video" },
            channel,
            title
        );

        let mut args = vec![
            &playlist.id,
            "-P",
            &out_path,
            "-q",
            "--no-warnings",
            "--progress",
            "--newline",
            "--progress-template",
            "download:%(info.playlist_index)s|%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.fragment_index)s|%(progress.fragment_count)s",
            "--output",
            "%(playlist_index)s - %(title)s [%(id)s].%(ext)s"
        ];

        if !Path::exists(Path::new(&out_path)) {
            let _ = std::fs::create_dir(&out_path);
        }

        let ext = self.selected_format.as_ext();
        let quality: String;
        let v_filter: String;

        if self.selected_format.is_audio() {
            args.extend([
                "-x",
                "--audio-format",
                ext
            ]);
        }
        else {
            let q = self.selected_quality.num().to_string();
            v_filter = format!("b[height={}]/bv[height={}]+ba", ext, q);
            quality = format!("res:{}", self.selected_quality.num().to_string());

            args.extend([
                "-S",
                &quality,
                "-f",
                &v_filter,
                "--remux-video",
                ext
            ]);
        }

        let command = match instance.create_download_process(&args) {
            Ok((mut stdout, stderr)) => {
                let mut output = String::new();
                let result = stdout.read_line(&mut output);

                self.downloading = true;
                self.download_info = Some(DownloadInfo::new(out_path, stdout, stderr));

                Task::done(Msg::NextVideoChunk(output, result.map_err(PomeloError::new)).into())
            },

            Err(e) => Task::done(Msg::VideoDownloadComplete(Err(e)).into())
        };

        (command, Navigation::None)
    }

    // Called when yt-dlp collects a chunk of bytes. Info from yt-dlp is used to update UI during download.
    fn on_next_chunk(&mut self, output: String, result: Result<usize, PomeloError>) -> (Task<Msg>, Navigation) {
        let command = match result {
            Ok(index) => match index {
                0 => Task::done(Msg::VideoDownloadComplete(Ok(())).into()),
                _ => {

                    let info = self.download_info.as_mut().unwrap();

                    // Read formatted progress string from yt-dlp
                    let nums: Vec<usize> = output
                        .trim()
                        .split('|')
                        .map(|s| s.parse().unwrap_or_default())
                        .collect();

                    self.download_index = nums[0];

                    if nums[2] != 0 {
                        info.progress = nums[1];
                        info.length = nums[2];
                    }

                    else {
                        info.progress = nums[3];
                        info.length = nums[4];
                    }

                    let mut output = String::new();
                    let result = info.stdout
                        .read_line(&mut output)
                        .map_err(PomeloError::new);

                    Task::done(Msg::NextVideoChunk(output, result).into())
                }
            },

            Err(e) => Task::done(Msg::VideoDownloadComplete(Err(e)).into())
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
            .height(instance.settings().window_size().1 / 2.0)
            .into()
    }
}

// The download was cancelled by the user.
fn on_download_cancelled(instance: &mut PomeloInstance) -> (Task<Msg>, Navigation) {
    instance.cancel_download();

    (
        Task::done(Msg::VideoDownloadComplete(Err(PomeloError::from("Cancelled by user."))).into()),
        Navigation::None
    )
}