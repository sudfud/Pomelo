use std::collections::VecDeque;
use std::time::Duration;
use std::num::Wrapping;

use rand::seq::SliceRandom;

use url::Url;

use log::{info, error};

use iced::Task;

use crate::INVID_INSTANCES;
use crate::app::PomeloError;
use iced_video_player::Video;

use super::{FillElement, PomeloInstance, Navigation, Msg};

#[derive(Debug, Clone)]
pub (crate) enum VideoPlayerMessage {
    LoadVideo,
    LoadComplete(Result<(Url, bool), PomeloError>),
    NextVideo(usize),
    PlayToggle,
    VolumeUpdate(f64),
    NextFrame,
    Seek(f64),
    SeekRelease
}

impl From<VideoPlayerMessage> for Msg {
    fn from(value: VideoPlayerMessage) -> Self {
        Self::VideoPlayer(value)
    }
}

impl super::ConditionalMessage for VideoPlayerMessage {}

// Plays a list of videos, either from the computer or from Youtube.
pub (crate) struct VideoPlayerPage {
    videos: VecDeque<(String, bool)>,
    video_index: Wrapping<usize>,
    current_video: Option<Result<Video, PomeloError>>,
    video_paused: bool,
    video_position: f64,
    video_volume: f64,
    seeking: bool,
}

impl super::PomeloPage for VideoPlayerPage {

    fn update(&mut self, instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation) {

        if let Msg::Back = message {
            return (Task::none(), Navigation::Back);
        }

        else if let Msg::VideoPlayer(msg) = message {
            match msg {

                VideoPlayerMessage::LoadVideo
                    => return self.load_video(instance.settings().invidious_index()),

                VideoPlayerMessage::LoadComplete(result) 
                    => self.on_load_complete(result),

                // Video control messages
                VideoPlayerMessage::NextVideo(index)
                    => return self.next_video(index),

                VideoPlayerMessage::PlayToggle 
                    => self.toggle_playback(),

                VideoPlayerMessage::VolumeUpdate(f) 
                    => self.set_volume(f),

                VideoPlayerMessage::Seek(f) 
                    => self.seek(f),

                VideoPlayerMessage::SeekRelease 
                    => self.on_seek_release(),

                // Keep track of the video position while it's playing
                VideoPlayerMessage::NextFrame 
                    => self.on_next_frame()
            }

        }

        (Task::none(), Navigation::None)
    }

    fn view(&self, _instance: &PomeloInstance) -> iced::Element<Msg> {
        use crate::utils;
        use iced::Length;
        use iced::widget::{row, Column, Text, Slider};
        use iced_video_player::VideoPlayer;
        use super::{centered_text_button, ConditionalMessage};

        if let Some(result) = &self.current_video {

            let mut column: Column<Msg> = Column::new()
                .spacing(10)
                .align_x(iced::Alignment::Center);

            match result {
                Ok(video) => {

                    let use_hour_timestamp = video.duration().as_secs() >= 3600;

                    let play_button_text = if self.is_video_playing() {
                        "Pause"
                    } else {
                        "Play"
                    };

                    let video_player = VideoPlayer::new(video)
                        .on_new_frame(VideoPlayerMessage::NextFrame.into())
                        .on_end_of_stream(
                            VideoPlayerMessage::NextVideo((self.video_index + Wrapping(1)).0).into()
                        );

                    // Add the video display
                    column = column.push(
                        video_player.fill()
                    );

                    // Add video controls
                    column = column.push(
                        row![

                            // Play/Pause button
                            centered_text_button(play_button_text, Some(100), None::<Length>)
                                .on_press(VideoPlayerMessage::PlayToggle.into()),

                            // Label for elapsed time
                            Text::new(
                                utils::secs_to_timestamp(
                                    self.video_position as u64,
                                    use_hour_timestamp
                                )
                            ),

                            // Playback slider
                            Slider::new(
                                0.0..=video.duration().as_secs_f64(),
                                self.video_position,
                                |f| VideoPlayerMessage::Seek(f).into()
                            ).step(0.1).on_release(VideoPlayerMessage::SeekRelease.into()),

                            // Label for total video length
                            Text::new(
                                utils::secs_to_timestamp(
                                    video.duration().as_secs(),
                                    use_hour_timestamp)
                                ),

                            Slider::new(
                                0.0..=1.0,
                                self.video_volume,
                                |f| VideoPlayerMessage::VolumeUpdate(f).into()
                            ).width(100).step(0.01)

                        ].spacing(10),
                    );
                },
                Err(e) => {
                    column = column.push(Text::new(e.error.to_string()));
                }
            }

            let buttons = row![

                centered_text_button("Prev", Some(100), None::<Length>)
                    .on_press_maybe(
                        VideoPlayerMessage::NextVideo((self.video_index - Wrapping(1)).0)
                        .on_condition(self.video_index.0 > 0)
                    ),

                centered_text_button("Back", Some(100), None::<Length>).on_press(Msg::Back),
                
                centered_text_button("Next", Some(100), None::<Length>)
                    .on_press_maybe(
                        VideoPlayerMessage::NextVideo((self.video_index+Wrapping(1)).0)
                            .on_condition(self.video_index.0 < self.videos.len())
                    ),

            ].spacing(25);

            column = column.push(buttons);

            return column.fill();
        }

        else {
            "Loading...".fill()
        }
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<Msg> {
        iced::Subscription::none()
    }
}

impl VideoPlayerPage {

    // Start loading the current video for playback.
    fn load_video(&self, instance_index: usize) -> (Task<Msg>, Navigation) {
        use crate::yt_fetch::VideoFetcher;

        let (video, from_computer) = self.videos[self.video_index.0].clone();

        info!("Loading video for playback: {}", video);

        if from_computer {
            (
                Task::done(
                    VideoPlayerMessage::LoadComplete(Ok((Url::parse(&video).unwrap(), false))).into()
                ),
                Navigation::None
            )
        }
        else {
            let instance = String::from(INVID_INSTANCES[instance_index].0);
            (
                Task::perform(
                    async move {
                        let downloader = VideoFetcher::new(instance);

                        match downloader.get_video_details(&video).await {
                            Ok(r) => Ok((Url::parse(&r.format_streams[0].url).unwrap(), r.live)),
                            Err(e) => Err(PomeloError::new(e))
                        }
                    },
                    
                    |result| VideoPlayerMessage::LoadComplete(result).into()
                ),
                Navigation::None
            )
        }
    }

    // Video finished loading, start playing if there were no errors.
    fn on_load_complete(&mut self, result: Result<(Url, bool), PomeloError>) {
        match result {
            Ok((url, live)) => {
                let video = match Video::new(&url, live) {
                    Ok(mut v) => {
                        let _ = v.seek(0);  // For some reason autoplay doesn't work properly without this line
                        v.set_volume(self.video_volume);
                        Ok(v)
                    },
                    Err(e) => {
                        error!("Failed to load video: {}", e);
                        Err(PomeloError::new(e))
                    }
                };
                self.current_video = Some(video);
            },

            Err(e) => {
                error!("Failed to load video: {}", e.error);
                self.current_video = Some(Err(e));
            }
        }
    }

    // Start loading the next video in the list.
    fn next_video(&mut self, index: usize) -> (Task<Msg>, Navigation) {
        if index < self.videos.len() {
            self.current_video = None;
            self.video_index = Wrapping(index);
            (
                Task::perform(
                    async {},
                    |_| VideoPlayerMessage::LoadVideo.into()
                ),
                Navigation::None
            )
        }
        else {
            (Task::none(), Navigation::None)
        }
    }

    // Pause/Play the video.
    fn toggle_playback(&mut self) {
        if let Some(Ok(video)) = self.current_video.as_mut() {
            video.set_paused(!video.paused());
            self.video_paused = video.paused();
        }
    }

    // Set the video's volume, 0.0 for mute, 1.0 for full volume.
    fn set_volume(&mut self, volume: f64) {
        if let Some(Ok(video)) = self.current_video.as_mut() {
            self.video_volume = volume;
            video.set_volume(volume);
        }
    }

    // Track the new position, and keep the video paused while seeking.
    fn seek(&mut self, position: f64) {
        if let Some(Ok(video)) = self.current_video.as_mut() {
            if !self.seeking {
                self.seeking = true;
                video.set_paused(true);
            }
            self.video_position = position;
        }
    }

    // Seek the video to the new position
    fn on_seek_release(&mut self) {
        if let Some(Ok(video)) = self.current_video.as_mut() {
            self.seeking = false;
            if let Err(e) =  video.seek(Duration::from_secs_f64(self.video_position)) {
                eprintln!("{}", e)
            }
            video.set_paused(self.video_paused);
        }
    }

    // Track the video's current position while it's playing.
    fn on_next_frame(&mut self) {
        if let Some(Ok(video)) = self.current_video.as_mut() {
            if !self.seeking {
                self.video_position = video.position().as_secs_f64();
            }
        }
    }
}

impl VideoPlayerPage {
    pub (crate) fn new(mut videos: VecDeque<(String, bool)>, order: super::VideoOrder) -> Self {
        use super::VideoOrder;

        let video_index = match order {
            VideoOrder::Sequential(index) => Wrapping(index),
            VideoOrder::Reversed => {
                videos.make_contiguous().reverse();
                Wrapping(0)
            },
            VideoOrder::Shuffled => {
                videos.make_contiguous().shuffle(&mut rand::thread_rng());
                Wrapping(0)
            }
        };

        Self {
            videos,
            video_index,
            current_video: None,
            video_paused: false,
            video_position: 0.0,
            video_volume: 0.5,
            seeking: false
        }
    }

    fn is_video_playing(&self) -> bool {
        if let Some(Ok(video)) = &self.current_video {
            return !video.paused();
        }
        false
    }
}