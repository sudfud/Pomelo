mod main_menu;
mod settings_page;
mod local_video_page;
mod video_player_page;
mod search_page;
mod search_results_page;
mod video_info_page;
mod playlist_info_page;

use std::io::BufReader;
use std::process::{ChildStderr, ChildStdout};

use iced::{Element, Length, Subscription, Task};

use crate::app::{DownloadFormat, DownloadQuality, PomeloError};
use crate::yt_fetch::{SearchResult, SearchResults};

use super::instance::cache::PomeloCache;
use super::instance::PomeloInstance;

pub (crate) use self::{
    main_menu::{MainMenuMessage, MainMenu},
    local_video_page::LocalVideoMessage,
    search_page::SearchMessage,
    search_results_page::SearchResultsMessage,
    video_info_page::VideoInfoMessage,
    playlist_info_page::PlaylistInfoMessage,
    video_player_page::VideoPlayerMessage,
    settings_page::SettingsMessage
};

type Msg = crate::app::PomeloMessage;

// Companion to Messages, used to redirect to different pages.
pub (crate) enum Navigation {
    GoTo(Box<dyn PomeloPage>),
    Back,
    Home,
    None
}

#[derive(Debug, Clone)]
pub (crate) enum VideoOrder {
    Sequential(usize),
    Reversed,
    Shuffled
}


// Allows pages to interact with the Iced update/render loops
pub (crate) trait PomeloPage {
    fn update(&mut self, instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation);
    fn view(&self, instance: &PomeloInstance) -> Element<Msg>;
    fn subscription(&self, instance: &PomeloInstance) -> Subscription<Msg>;
}

// Convenience trait for expanding UI elements to fit the whole screen.
trait FillElement<'a, T> {
    fn fill(self) -> Element<'a, Msg>;
}

impl <'a, T> FillElement<'a, T> for T where T: Into<Element<'a, Msg>> {
    fn fill(self) -> Element<'a, Msg> {
        iced::widget::Container::new(self)
            .center(Length::Fill)
            .into()
    }
}

trait ConditionalElement<'a> {
    fn on_condition(self, condition: bool) -> Option<Element<'a, Msg>> where Self: Into<Element<'a, Msg>> {
        if condition {
            Some(self.into())
        }
        else {
            None
        }
    }
}

impl <'a> ConditionalElement<'a> for iced::widget::Button<'a, Msg> {}
impl <'a> ConditionalElement<'a> for iced::Element<'a, Msg> {}

// Convenience trait for optional messages
trait ConditionalMessage {
    fn on_condition(self, condition: bool) -> Option<Msg> where Self: Into<Msg> {
        if condition {
            Some(self.into())
        }
        else {
            None
        }
    }
}

impl ConditionalMessage for Msg {}

// Collection of information and readers for a video/playlist download.
// Might want to move up to app module later, and make this a part of PomeloInstance
struct DownloadInfo {
    path: String,
    stdout: BufReader<ChildStdout>,
    stderr: BufReader<ChildStderr>,
    progress: usize,
    length: usize
}

impl DownloadInfo {
    fn new(path: String, stdout: BufReader<ChildStdout>, stderr: BufReader<ChildStderr>) -> Self {
        Self {
            path,
            stdout,
            stderr,
            progress: 0,
            length: 0
        }
    }
}

fn download_element<'a>(format: &'a DownloadFormat, quality: &'a DownloadQuality) -> iced::Element<'a, Msg> {
    use iced::widget::{column, Row, Button, Text};

    let mut row = Row::new().spacing(10);

    row = row.push(
        labeled_picklist(
            "Format",
            DownloadFormat::ALL,
            format.clone(),
            |fmt| Msg::SetDownloadFormat(fmt).into()
        )  
    );

    row = row.push_maybe(
        labeled_picklist(
            "Quality",
            DownloadQuality::ALL,
            quality.clone(),
            |q| Msg::SetDownloadQuality(q).into()
        ).on_condition(!format.is_audio())
    );

    column![
        Button::new(Text::new("Download").center())
            .width(100)
            .on_press(Msg::StartVideoDownload.into()),

        row

    ].align_x(iced::Alignment::Center).into()
}

fn labeled_picklist<'a, L, T, V>(text: &'a str, list: L, select: V, on_select: impl Fn(T) -> Msg + 'a) -> iced::Element<Msg> 
    where 
        L: std::borrow::Borrow<[T]> + 'a,
        T: ToString + PartialEq + Clone + 'a,
        V: std::borrow::Borrow<T> + 'a
{
    use iced::Alignment;
    use iced::widget::{column, PickList, Text};

    column![
        Text::new(text),
        PickList::new(list, Some(select), on_select).width(200)
    ].spacing(5).align_x(Alignment::Center).into()
}

// Load thumbnails asyncronously
fn batch_thumbnail_commands(search: &SearchResults, cache: &PomeloCache) -> Task<Msg> {
    use crate::yt_fetch::download_thumbnail;

    let mut commands: Vec<Task<Msg>> = Vec::new();
    
    for item in search.get_results().into_iter() {
        let id = match &item {
            SearchResult::Video(video) => video.id.clone(),
            SearchResult::Channel(channel) => channel.id.clone(),
            SearchResult::Playlist(playlist) => playlist.id.clone(),
            SearchResult::PlaylistVideo(video) => video.id.clone()
        };

        if !cache.has_thumbnail(&id) {
            commands.push(Task::perform(
                async move {
                    (id, download_thumbnail(&item, 4).await)
                },
                
                |(id, result)| {
                    let out = match result {
                        Ok(handle) => Ok((id, handle)),
                        Err(e) => Err(PomeloError::new(e))
                    };
                    Msg::ThumbnailLoaded(out)
                }
            ));
        }
    }

    Task::batch(commands)
}