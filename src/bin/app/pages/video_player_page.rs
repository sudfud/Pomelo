use std::collections::VecDeque;
use std::time::Duration;
use std::num::Wrapping;

use rand::seq::SliceRandom;

use url::Url;

use log::{info, error};

use iced::Task;

use crate::app::pages::ConditionalElement;
use crate::INVID_INSTANCES;
use crate::app::PomeloError;
use iced_video_player::Video;

use super::{FillElement, PomeloInstance, Navigation, Msg};

#[derive(Debug, Clone)]
pub (crate) enum VideoPlayerMessage {
    LoadVideo(usize),
    LoadComplete(usize, Result<(Url, bool), PomeloError>),
    NextVideo(usize),
    PlayToggle,
    VolumeUpdate(f64),
    NextFrame,
    Seek(f64),
    SeekRelease,
    SkipTimer(u8, usize)
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
    skip_timer: Option<iced::task::Handle>,
    auto_skipping: bool,
    skip_time: u8
}

impl super::PomeloPage for VideoPlayerPage {

    fn update(&mut self, instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation) {

        if let Msg::Back = message {
            if let Some(timer) = self.skip_timer.take() {
                timer.abort();
            }

            return (Task::none(), Navigation::Back);
        }

        else if let Msg::VideoPlayer(msg) = message {
            match msg {
                VideoPlayerMessage::LoadVideo(index) => return (
                    self.load_video(index, instance),
                    Navigation::None
                ),

                VideoPlayerMessage::LoadComplete(index, result) => return (
                    self.on_load_complete(index, result, instance.settings().video_skip_on_error()),
                    Navigation::None
                ),

                // Video control messages
                VideoPlayerMessage::NextVideo(index) => return (
                    self.next_video(index),
                    Navigation::None
                ),

                
                VideoPlayerMessage::SkipTimer(time, index) => return (
                    self.skip_timer_update(time, index),
                    Navigation::None
                ),

                VideoPlayerMessage::PlayToggle => self.toggle_playback(),
                VideoPlayerMessage::VolumeUpdate(f) => self.set_volume(f),
                VideoPlayerMessage::Seek(f) => self.seek(f),
                VideoPlayerMessage::SeekRelease => self.on_seek_release(),
                VideoPlayerMessage::NextFrame => self.on_next_frame()
            }
        }

        (Task::none(), Navigation::None)
    }

    fn view(&self, _instance: &PomeloInstance) -> iced::Element<Msg> {
        use crate::utils;
        use iced::widget::{row, Row, Column, Text, Slider, Button};
        use iced_video_player::VideoPlayer;
        use super::ConditionalMessage;

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
                            Button::new(Text::new(play_button_text).center())
                                .width(100)
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

                        ].spacing(10)
                    );
                },
                Err(e) => {
                    let error_msg = e.error.to_string();
                    column = column.push(Text::new(error_msg).center());
                    if self.auto_skipping {
                        let skip_str = format!("Skipping in {}", self.skip_time);
                        column = column.push(Text::new(skip_str).center())
                    }
                }
            }

            let mut buttons = Row::<Msg>::new().spacing(25);

            buttons = buttons.push_maybe(
                Button::new(Text::new("Prev").center())
                    .width(100)
                    .on_press_maybe(
                        VideoPlayerMessage::NextVideo((self.video_index - Wrapping(1)).0)
                        .on_condition(self.video_index.0 > 0))
                    .on_condition(self.videos.len() > 1)
            );

            buttons = buttons.push(
                Button::new(Text::new("Back").center())
                    .width(100)
                    .on_press(Msg::Back)
            );

            buttons = buttons.push_maybe(
                Button::new(Text::new("Next").center())
                    .width(100)
                    .on_press_maybe(
                        VideoPlayerMessage::NextVideo((self.video_index+Wrapping(1)).0)
                            .on_condition(self.video_index.0 < self.videos.len()-1))
                    .on_condition(self.videos.len() > 1)
            );

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
    fn load_video(&self, video_index: usize, instance: &PomeloInstance) -> Task<Msg> {
        use crate::yt_fetch::VideoFetcher;

        let (video, from_computer) = self.videos[video_index].clone();

        info!("Loading video for playback: {}", video);

        let invid_index = String::from(INVID_INSTANCES[instance.settings().invidious_index()].0);

        Task::perform(
            async move {
                if from_computer {
                    Url::parse(&video)
                        .map(|url| (url, false))
                        .map_err(|e| {
                                eprintln!("{}", e);
                                PomeloError::new(e)
                            }
                        )
                } 
                else {
                    let downloader = VideoFetcher::new(invid_index);
                    
                    match downloader.get_video_details(&video).await {
                        Ok(r) => Url::parse(&r.format_streams[0].url)
                            .map(|url| (url, r.live))
                            .map_err(PomeloError::new),

                        Err(e) => Err(PomeloError::new(e))
                    }
                }
            },
            move |result| VideoPlayerMessage::LoadComplete(video_index, result).into()
        )
    }

    // Video finished loading, start playing if there were no errors.
    fn on_load_complete(&mut self, video_index: usize, result: Result<(Url, bool), PomeloError>, skip_on_error: bool) -> Task<Msg> {
        let mut maybe_video = match result {
            Ok((url, live)) => Video::new(&url, live).map_err(PomeloError::new),
            Err(e) => {
                Err(e)
            }
        };

        let task = match &mut maybe_video {
            Ok(video) => {
                self.video_index = Wrapping(video_index);
                let _ = video.seek(0);  // For some reason autoplay doesn't work properly without this line
                video.set_volume(self.video_volume);
                Task::none()
            },

            Err(e) => {
                error!("Failed to load video: {}", e.error);

                if skip_on_error && !(video_index == 0 || video_index == self.videos.len()-1) {

                    let next_index = if self.video_index.0 <= video_index {
                        video_index + 1
                    } else if video_index > 0 {
                        video_index - 1
                    } else {
                        0
                    };
                    
                    self.video_index = Wrapping(video_index);
                    self.auto_skipping = true;

                    let (timer, handle) = Task::done(
                        VideoPlayerMessage::SkipTimer(5, next_index).into()
                    ).abortable();

                    self.skip_timer = Some(handle);

                    timer
                }
                else {
                    self.video_index = Wrapping(video_index);
                    Task::none()
                }
            }
        };

        self.current_video = Some(maybe_video);

        task
    }

    // Start loading the next video in the list.
    fn next_video(&mut self, index: usize) -> Task<Msg> {

        if let Some(handle) = self.skip_timer.take() {
            handle.abort();
        }

        if index > self.video_index.0 && index < self.videos.len() ||
            index < self.video_index.0 && index > 0 
        {
            self.current_video = None;
            //self.video_index = Wrapping(index);

            Task::done(VideoPlayerMessage::LoadVideo(index).into())
        }
        else {
            Task::none()
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

    fn skip_timer_update(&mut self, time: u8, index: usize) -> Task<Msg> {
        self.skip_time = time;

        if time == 0 {
            self.auto_skipping = false;
            Task::done(VideoPlayerMessage::NextVideo(index).into())
        }
        else {

            Task::perform(
                async {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                },
                move |_| VideoPlayerMessage::SkipTimer(time - 1, index).into()
            )
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
            seeking: false,
            skip_timer: None,
            auto_skipping: false,
            skip_time: 0
        }
    }

    fn is_video_playing(&self) -> bool {
        if let Some(Ok(video)) = &self.current_video {
            return !video.paused();
        }
        false
    }
}