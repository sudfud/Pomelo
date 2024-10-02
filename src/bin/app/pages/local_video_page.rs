use iced::Task;

use crate::app::{PomeloInstance, PomeloMessage, PomeloCommand};

use super::{VideoOrder, Navigation};

#[derive(Debug, Clone)]
pub (crate) enum LocalVideoMessage {
    OpenFilePicker,
    PlayVideos(VideoOrder),
    ClearVideos
}

impl From<LocalVideoMessage> for PomeloMessage {
    fn from(value: LocalVideoMessage) -> Self {
        Self::LocalVideo(value)
    }
}

// A page for the user to load videos directly from their computer, with options for playback
pub (crate) struct LocalVideoPage {
    videos: Vec<String>
}

impl super::PomeloPage for LocalVideoPage {
    fn update(&mut self, _instance: &mut PomeloInstance, message: PomeloMessage) -> PomeloCommand {
        if let PomeloMessage::Back = message {
            return PomeloCommand::back();
        }

        if let PomeloMessage::LocalVideo(msg) = message {
            match msg {
                LocalVideoMessage::OpenFilePicker => return self.open_file_picker(),
                LocalVideoMessage::PlayVideos(order) => return self.play_videos(order),
                LocalVideoMessage::ClearVideos => self.videos.clear()
            }
        }

        PomeloCommand::none()
    }

    fn view(&self, instance: &PomeloInstance) -> iced::Element<PomeloMessage> {
        use iced::Element;
        use iced::widget::{column, row, Column, Scrollable, Text};
        use super::{FillElement, simple_button};

        let video_list: Vec<Element<PomeloMessage>> = self.videos.iter()
            .map(|s| Text::new(s.split('/').last().unwrap()).into())
            .collect();

        column![
            if self.videos.is_empty() {
                simple_button("Load Videos", 200, LocalVideoMessage::OpenFilePicker)
            } 
            else {
                Element::<PomeloMessage>::from(
                    column![
                        Scrollable::new(Column::from_vec(video_list))
                            .height(instance.settings().window_size().1 / 2.0),

                        row![
                            simple_button("Play", 100, LocalVideoMessage::PlayVideos(VideoOrder::Sequential(0))),
                            simple_button("Shuffle", 100, LocalVideoMessage::PlayVideos(VideoOrder::Shuffled)),
                            simple_button("Reverse", 100, LocalVideoMessage::PlayVideos(VideoOrder::Reversed))
                        ].spacing(10),

                        simple_button("Clear", 100, LocalVideoMessage::ClearVideos)

                    ].spacing(25).align_x(iced::Alignment::Center)
                )
            },

            simple_button("Back", 100, PomeloMessage::Back)
        ].spacing(25).align_x(iced::Alignment::Center).fill()
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<PomeloMessage> {
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
    fn open_file_picker(&mut self) -> PomeloCommand {
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

        PomeloCommand::none()
    }

    fn play_videos(&self, order: VideoOrder) -> PomeloCommand {
        use std::collections::VecDeque;
        use super::video_player_page::{VideoPlayerMessage, VideoPlayerPage};

        let vids: VecDeque<(String, bool)> = self.videos.iter()
            .map(|s| (String::from(s), true))
            .collect();

        let index = if let VideoOrder::Sequential(i) = order {i} else {0};

        PomeloCommand::go_to_with_message(VideoPlayerMessage::LoadVideo(index), VideoPlayerPage::new(vids, order))
    }
}