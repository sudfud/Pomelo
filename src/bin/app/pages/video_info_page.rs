use std::collections::VecDeque;
use std::io::BufRead;

use iced::Task;

use invidious::CommonVideo;

use log::{info, error};

use crate::INVID_INSTANCES;
use crate::app::{DownloadFormat, DownloadQuality, PomeloError};
use crate::yt_fetch::VideoFetcher;

use super::{DownloadInfo, PomeloInstance, Navigation, Msg};

#[derive(Debug, Clone)]
pub (crate) enum VideoInfoMessage {
    LoadVideo(String),
    VideoLoaded(Result<CommonVideo, PomeloError>),
    PlayVideo,
    // SetDownloadFormat(DownloadFormat),
    // SetDownloadQuality(DownloadQuality),
    // DownloadVideo,
    // NextChunk(String, Result<usize, PomeloError>),
    // DownloadCancelled,
    // DownloadComplete(Result<(), PomeloError>)
}

impl From<VideoInfoMessage> for Msg {
    fn from(value: VideoInfoMessage) -> Self {
        Self::VideoInfo(value)
    }
}

impl super::ConditionalMessage for VideoInfoMessage {}

// Displays info for a given video, with options for playback and downloading.
#[derive(Default)]
pub (crate) struct VideoInfoPage {
    video: Option<CommonVideo>,
    downloading: bool,
    selected_format: DownloadFormat,
    selected_quality: DownloadQuality,
    download_info: Option<DownloadInfo>,
    download_error: Option<PomeloError>
}

impl VideoInfoPage {
    pub (crate) fn new() -> Self {
        Default::default()
    }

    pub (crate) fn new_with_video(video: CommonVideo) -> Self {
        Self {
            video: Some(video),
            ..Default::default()
        }
    }
}

impl super::PomeloPage for VideoInfoPage {
    fn update(&mut self, instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation) {

        match message {
            Msg::Back => return (Task::none(), Navigation::Back),
            Msg::SetDownloadFormat(format) => self.selected_format = format,
            Msg::SetDownloadQuality(quality) => self.selected_quality = quality,
            Msg::StartDownload => return self.download_video(instance),
            Msg::NextChunk(line, result) => return self.on_next_chunk(line, result),
            Msg::DownloadCancelled => return on_download_cancelled(instance),
            Msg::DownloadComplete(result) => self.on_download_complete(result),

            Msg::VideoInfo(msg) => match msg {
                VideoInfoMessage::LoadVideo(id) 
                    => return load_video(id, instance.settings().invidious_index()),

                VideoInfoMessage::VideoLoaded(result)
                    => return self.on_video_loaded(result),

                VideoInfoMessage::PlayVideo
                    => return self.play_video()
            }

            _ => ()
        }

        // if let Msg::Back = message {
        //     return (
        //         Task::none(),
        //         Navigation::Back
        //     );
        // }

        // if let Msg::VideoInfo(msg) = message {
        //     match msg {
        //         VideoInfoMessage::LoadVideo(id) 
        //             => return load_video(id, instance.settings().invidious_index()),

        //         VideoInfoMessage::VideoLoaded(result) 
        //             => return self.on_video_loaded(result),

        //         VideoInfoMessage::PlayVideo 
        //             => return self.play_video(),

        //         VideoInfoMessage::SetDownloadFormat(format)
        //             => self.selected_format = format,

        //         VideoInfoMessage::SetDownloadQuality(quality)
        //             => self.selected_quality = quality,

        //         VideoInfoMessage::DownloadVideo 
        //             => return self.download_video(instance),

        //         VideoInfoMessage::NextChunk(line, result) 
        //             => return self.on_next_chunk(line, result),
                    
        //         VideoInfoMessage::DownloadComplete(result) 
        //             => self.on_download_complete(result),

        //         VideoInfoMessage::DownloadCancelled
        //             => return on_download_cancelled(instance)
        //     }
        // }

        (Task::none(), Navigation::None)
    }

    fn view(&self, instance: &PomeloInstance) -> iced::Element<Msg> {
        use iced::{Alignment, Length};
        use iced::widget::{row, column, Column, Image, ProgressBar, Button, Text};
        use super::{centered_text_button, download_element, FillElement};

        match &self.video {
            Some(video) => {
                let mut column: Column<Msg> = Column::new()
                .spacing(25)
                .align_x(iced::Alignment::Center);
    
                if let Some(handle) = instance.cache().get_thumbnail(&video.id) {
                    column = column.push(Image::new(handle.clone()));
                }
        
                column = column.push(
                    column![
                        Text::new(video.title.clone()),
                        Text::new(format!("{}\n", video.author)),
                        Text::new(format!("{} Views", video.views))
                    ]
                );
        
                if let Some(e) = &self.download_error {
                    column = column.push(Text::new(&e.error));
                }

                // Draw download progress.
                if self.downloading {
                    let info = self.download_info.as_ref().unwrap();
                    column = column.extend(
                        vec![
                            ProgressBar::new(0.0..=info.length as f32, info.progress as f32)
                                .width(instance.settings().window_size().0 / 2.0)
                                .into(),
        
                            Button::new("Cancel")
                                .width(200)
                                .on_press(Msg::DownloadCancelled.into())
                                .into()
                        ]
                    );
                }

                // Draw playback, download, and navigation buttons.
                else {
                    column = column.push(
                        column![
                            centered_text_button("Play", Some(100), None::<Length>)
                                .on_press(VideoInfoMessage::PlayVideo.into()),

                            download_element(&self.selected_format, &self.selected_quality),

                            // column![
                            //     centered_text_button("Download", Some(100), None::<Length>)
                            //         .on_press(Msg::StartDownload.into()),

                            //     row![
                            //         labeled_picklist(
                            //             "Format",
                            //             DownloadFormat::ALL,
                            //             self.selected_format.clone(),
                            //             |fmt| Msg::SetDownloadFormat(fmt).into()
                            //         ),

                            //         labeled_picklist(
                            //             "Quality",
                            //             DownloadQuality::ALL,
                            //             self.selected_quality.clone(),
                            //             |q| Msg::SetDownloadQuality(q).into()
                            //         )
                            //     ].spacing(10),

                            // ].align_x(Alignment::Center),

                            centered_text_button("Back", Some(100), None::<Length>)
                                .on_press(Msg::Back)

                        ].spacing(50).align_x(Alignment::Center)
                    );
                }
        
                column.fill()
            },
            None => Text::new("Loading...").fill()
        }
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<Msg> {
        iced::Subscription::none()
    }
}

impl VideoInfoPage {
    // Video finished loading, or an error occured.
    fn on_video_loaded(&mut self, result: Result<CommonVideo, PomeloError>) -> (Task<Msg>, Navigation) {
        use crate::yt_fetch::{SearchResult, download_thumbnail};

        let command = match result {
            Ok(video) => {
                info!("Info load complete.");
                self.video = Some(video.clone());
                Task::perform(
                    async {
                        let id = video.id.clone();
                        download_thumbnail(&SearchResult::Video(video), 4).await
                            .map(|handle| (id, handle))
                            .map_err(PomeloError::new)
                    },
                    Msg::ThumbnailLoaded
                )
            },
            Err(e) => {
                error!("Failed to load video info: {}", e.error);
                self.download_error = Some(e);
                Task::none()
            }
        };

        (command, Navigation::None)
    }

    // Move video player page.
    fn play_video(&self) -> (Task<Msg>, Navigation) {
        use super::VideoOrder;
        use super::video_player_page::{VideoPlayerMessage, VideoPlayerPage};

        let id = self.video.as_ref().unwrap().id.clone();
        (
            Task::done(VideoPlayerMessage::LoadVideo(0).into()),
            Navigation::GoTo(
                Box::new(
                    VideoPlayerPage::new(VecDeque::from([(id, false)]), VideoOrder::Sequential(0))
                )
            )
        )
    }

    // Setup yt-dlp to download the video.
    fn download_video(&mut self, instance: &mut PomeloInstance) -> (Task<Msg>, Navigation) {
        use std::io::BufRead;
        use std::path::Path;

        let video = self.video.as_ref().unwrap();
        let out_path = format!(
            "./downloads/{}/{}",
            if self.selected_format.is_audio() {"audio"} else {"videos"},
            video.author
        );

        info!("Downloading video: \"{}\"", video.title);

        if !Path::exists(Path::new(&out_path)) {
            let _ = std::fs::create_dir(&out_path);
        }

        let mut args = vec![
            &video.id,
            "-P",
            &out_path,
            "-q",
            "-w",
            "--no-warnings",
            "--progress",
            "--newline",
            "--progress-template",
            "download:%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.fragment_index)s|%(progress.fragment_count)s",
            "--ffmpeg-location",
            "./ffmpeg/bin"
        ];

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
            println!("{}", quality);

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

                Task::done(
                    Msg::NextChunk(output, result.map_err(PomeloError::new)).into()
                )
            },

            Err(e) => Task::done(Msg::DownloadComplete(Err(e)).into())
        };

        (command, Navigation::None)
    }

    // Load the next chunk of bytes and append it to the video file
    fn on_next_chunk(&mut self, line: String, result: Result<usize, PomeloError>) -> (Task<Msg>, Navigation) {

        if line.to_lowercase().contains("error") {
            return (
                Task::done(
                    Msg::DownloadComplete(
                        Err(PomeloError::from(String::from("Failed to retrieve next video chunk.")))
                    ).into()
                ),

                Navigation::None
            );
        }

        let command = match result {
            Ok(index) => match index {
                0 => Task::done(Msg::DownloadComplete(Ok(())).into()),
                _ => {

                    let nums: Vec<usize> = line
                        .trim()
                        .split('|')
                        .map(|s| s.parse().unwrap_or_default())
                        .collect();
                    println!("{:?}", nums);

                    let info = self.download_info.as_mut().unwrap();

                    // Update progress bar, fallback to fragments if total_bytes is 0.
                    if nums[1] != 0 {
                        info.progress = nums[0];
                        info.length = nums[1];
                    }
                    else {
                        info.progress = nums[2];
                        info.length = nums[3];
                    }

                    let mut output = String::new();
                    let result = info.stdout
                        .read_line(&mut output)
                        .map_err(PomeloError::new);

                    Task::done(Msg::NextChunk(output, result).into())
                }
            },

            Err(e) => Task::done(Msg::DownloadComplete(Err(e)).into())
        };

        (command, Navigation::None)
    }

    // Video finished downloading, or an error occured.
    fn on_download_complete(&mut self, result: Result<(), PomeloError>) {
        use std::path::Path;

        if let Err(e) = result {
            error!("Download failed: {}", e.error);
            self.download_error = Some(e);
        }

        else {
            let info = self.download_info.take().unwrap();
            
            println!("Download complete!");

            if let Some(Ok(line)) = info.stderr.lines().last() {
                error!("Download failed: {}", line);
                self.download_error = Some(PomeloError::from(line));
            }

            else {
                info!("Video downloaded to file: {:?}", Path::new(&info.path));
            }
        }

        self.downloading = false;
    }
}

// Use Invidious to load video info from Youtube.
fn load_video(id: String, instance_index: usize) -> (Task<Msg>, Navigation) {
    info!("Loading video info with id: {}", id);

    let instance = String::from(INVID_INSTANCES[instance_index].0);
    (
        Task::perform(
            async move {
                let downloader = VideoFetcher::new(instance);

                downloader.get_video_details(&id)
                    .await
                    .map(|video| video.into())
                    .map_err(PomeloError::new)
            },
            |result| VideoInfoMessage::VideoLoaded(result).into()
        ),
        Navigation::None
    )
}

// Download was cancelled by the user.
fn on_download_cancelled(instance: &mut PomeloInstance) -> (Task<Msg>, Navigation) {
    instance.cancel_download();
    println!("Download cancelled!");
    (
        Task::done(Msg::DownloadComplete(Err(PomeloError::from("Cancelled by user."))).into()),
        Navigation::None
    )
}