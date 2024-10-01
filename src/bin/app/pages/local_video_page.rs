use iced::Task;

use crate::app::PomeloInstance;

use super::{VideoOrder, Navigation, Msg};

#[derive(Debug, Clone)]
pub (crate) enum LocalVideoMessage {
    OpenFilePicker,
    PlayVideos(VideoOrder),
    ClearVideos
}

impl From<LocalVideoMessage> for Msg {
    fn from(value: LocalVideoMessage) -> Self {
        Self::LocalVideo(value)
    }
}

// A page for the user to load videos directly from their computer, with options for playback
pub (crate) struct LocalVideoPage {
    videos: Vec<String>
}

impl super::PomeloPage for LocalVideoPage {
    fn update(&mut self, _instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation) {
        if let Msg::Back = message {
            return (Task::none(), Navigation::Back);
        }

        if let Msg::LocalVideo(msg) = message {
            match msg {
                LocalVideoMessage::OpenFilePicker => return self.open_file_picker(),
                LocalVideoMessage::PlayVideos(order) => return self.play_videos(order),
                LocalVideoMessage::ClearVideos => self.videos.clear()
            }
        }

        (Task::none(), Navigation::None)
    }

    fn view(&self, instance: &PomeloInstance) -> iced::Element<Msg> {
        use iced::Element;
        use iced::widget::{column, row, Column, Scrollable, Text, Button};
        use super::FillElement;

        let video_list: Vec<Element<Msg>> = self.videos.iter()
            .map(|s| Text::new(s.split('/').last().unwrap()).into())
            .collect();

        column![
            if self.videos.is_empty() {
                Element::<Msg>::from(
                    Button::new(Text::new("Load Videos").center())
                        .width(200)
                        .on_press(LocalVideoMessage::OpenFilePicker.into())
                )
            } 
            else {
                Element::<Msg>::from(
                    column![
                        Scrollable::new(Column::from_vec(video_list))
                            .height(instance.settings().window_size().1 / 2.0),

                        row![
                            Button::new(Text::new("Play").center())
                                .width(100)
                                .on_press(
                                    LocalVideoMessage::PlayVideos(VideoOrder::Sequential(0)).into()
                                ),

                            Button::new(Text::new("Shuffle").center())
                                .width(100)
                                .on_press(
                                    LocalVideoMessage::PlayVideos(VideoOrder::Shuffled).into()
                                ),

                            Button::new(Text::new("Reverse").center())
                                .width(100)
                                .on_press(
                                    LocalVideoMessage::PlayVideos(VideoOrder::Reversed).into()
                                )
                        ].spacing(10),

                        Button::new(Text::new("Clear").center())
                            .width(100)
                            .on_press(LocalVideoMessage::ClearVideos.into())

                    ].spacing(25).align_x(iced::Alignment::Center)
                )
            },

            Button::new(Text::new("Back").center())
                .width(100)
                .on_press(Msg::Back)
        ].spacing(25).align_x(iced::Alignment::Center).fill()
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<Msg> {
        iced::Subscription::none()
    }
}

impl LocalVideoPage {
    pub (crate) fn new() -> Self {
        Self {
            videos: Vec::new()
        }
    }

    // Select videos from the computer, then move them to the Video Player page.
    fn open_file_picker(&mut self) -> (Task<Msg>, Navigation) {
        use rfd::FileDialog;

        let maybe_files = FileDialog::new()
            .add_filter("video", &["mp4", "webm"])
            .set_directory(".")
            .pick_files();

        if let Some(files) = maybe_files {
            for file in files.into_iter() {
                self.videos.push(
                    format!("file:///{}", file.as_path().to_str().unwrap()).replace('\\', "/")
                );
            }
        }

        (Task::none(), Navigation::None)
    }

    fn play_videos(&self, order: VideoOrder) -> (Task<Msg>, Navigation) {
        use std::collections::VecDeque;
        use super::video_player_page::{VideoPlayerMessage, VideoPlayerPage};

        let vids: VecDeque<(String, bool)> = self.videos.iter()
            .map(|s| (String::from(s), true))
            .collect();

        let index = if let VideoOrder::Sequential(i) = order {i} else {0};

        (
            Task::done(VideoPlayerMessage::LoadVideo(index).into()),
            Navigation::GoTo(Box::new(VideoPlayerPage::new(vids, order)))
        )
    }
}