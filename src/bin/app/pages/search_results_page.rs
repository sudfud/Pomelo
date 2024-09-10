use std::collections::HashMap;

use iced::{Task, Length, Element};
use iced::widget::{column, row, Column, Row, Text, Button, Image};
use iced::widget::image::Handle;
use invidious::CommonVideo;
use log::{info, error};


use crate::INVID_INSTANCES;
use crate::app::PomeloError;
use crate::app::instance::cache::PomeloCache;
use crate::yt_fetch::{SearchResult, SearchResults, SearchType, VideoFetcher};

use super::{FillElement, PomeloInstance, Navigation, Msg};

// Convenience trait for grabbing info about a search item.
// Playlist videos are handled on a separate page, so they're listed as unreachable here.
trait YoutubeInfo {
    fn id(&self) -> String;
    fn info(&self) -> Vec<String>;
}

impl YoutubeInfo for SearchResult {
    fn id(&self) -> String {
        match self {
            Self::Video(v) => v.id.clone(),
            Self::Channel(ch) => ch.id.clone(),
            Self::Playlist(p) => p.id.clone(),
            Self::PlaylistVideo(_) => unreachable!()
        }
    }

    fn info(&self) -> Vec<String> {
        match self {
            Self::Video(v) => vec![
                v.title.clone(),
                v.author.clone(),
                format!("{} Views", v.views)
            ],
            Self::Channel(ch) => vec![
                ch.name.clone(),
                ch.description.clone(),
                format!("{} Subscribers", ch.subscribers),
                format!("{} Videos", ch.video_count)
            ],
            Self::Playlist(p) => vec![
                p.title.clone(),
                p.author.clone(),
                format!("{} Videos", p.video_count)
            ],
           Self::PlaylistVideo(_) => unreachable!()
        }
    }
}

#[derive(Debug, Clone)]
pub (crate) enum SearchResultsMessage {
    StartSearch,
    SearchComplete(Result<SearchResults, PomeloError>),
    NewPage(usize),
    ToVideo(CommonVideo),
    ToChannelVideos(String),
    ToPlaylistVideos(String)
}

impl From<SearchResultsMessage> for Msg {
    fn from(value: SearchResultsMessage) -> Self {
        Self::SearchResults(value)
    }
}

impl super::ConditionalMessage for SearchResultsMessage {}

// Displays the results of a search query.
// Redirects to itself when the user selects a channel or navigates to another search page.
// Redirects to video or playlist info page when the user selects a video/playlist.
pub (crate) struct SearchResultsPage {
    query: String,
    search_type: SearchType,
    search_results: Option<Result<SearchResults, PomeloError>>,
    page_number: usize,
    continuation: HashMap<usize, String>
}

impl super::PomeloPage for SearchResultsPage {
    fn update(&mut self, instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation) {
        if let Msg::Back = message {
            return (Task::none(), Navigation::Back);
        }

        else if let Msg::Home = message {
            return (Task::none(), Navigation::Home);
        }

        else if let Msg::SearchResults(msg) = message {
            match msg {
                SearchResultsMessage::StartSearch 
                    => return self.start_search(instance.settings().invidious_index()),

                SearchResultsMessage::SearchComplete(result) 
                    => return self.on_search_complete(result, instance.cache()),

                SearchResultsMessage::NewPage(page_number) 
                    => return self.on_new_page(page_number),

                SearchResultsMessage::ToVideo(id) 
                    => return go_to_video(id),

                SearchResultsMessage::ToChannelVideos(id)
                    => return go_to_channel_videos(&id),

                SearchResultsMessage::ToPlaylistVideos(id)
                    => return go_to_playlist_videos(id)
            }
        }

        (Task::none(), Navigation::None)
    }

    fn view(&self, instance: &PomeloInstance) -> Element<Msg> {
        use super::ConditionalMessage;

        if let Some(result) = &self.search_results {
            let result_element = self.get_search_results_element(result, instance);

            let buttons = row![
                Button::new(Text::new("Prev").center())
                    .width(100)
                    .on_press_maybe(
                        SearchResultsMessage::NewPage(self.page_number-1)
                            .on_condition(self.page_number > 1)
                    ),
            
                Button::new(Text::new("Back").center())
                    .width(100)
                    .on_press(Msg::Back),
            
                Button::new(Text::new("Next").center())
                    .width(100)
                    .on_press(SearchResultsMessage::NewPage(self.page_number+1).into())
            
            ].spacing(25);
    
            column![
                result_element,
                buttons,
                Button::new(Text::new("Home").center())
                    .width(100)
                    .on_press(Msg::Home)
            ].align_x(iced::Alignment::Center).spacing(25).into()
        }
        else {
            "Loading...".fill()
        }
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<Msg> {
        iced::Subscription::none()
    }
}

impl SearchResultsPage {

    pub (crate) fn new(query: String, search_type: SearchType) -> Self {
        Self {
            query,
            search_type,
            search_results: None,
            page_number: 1,
            continuation: HashMap::new()
        }
    }

    // Use Invidious to search for items from Youtube.
    fn start_search(&self, instance_index: usize) -> (Task<Msg>, Navigation) {
        let query = self.query.clone();
        let search_type = self.search_type;
        let page_number = self.page_number;
        let continuation = self.continuation.get(&self.page_number).cloned();
        let instance = String::from(INVID_INSTANCES[instance_index].0);

        info!("Starting Youtube search. Type: {}, Page: {}, Query: {}", search_type, page_number, query);
        
        (
            Task::perform(
                async move {
                    let downloader = VideoFetcher::new(instance);

                    if let SearchType::ChannelUploads = search_type {
                        println!("{:?}", continuation);
                        downloader.get_channel_videos(&query, continuation.as_deref()).await
                            .map(SearchResults::ChannelUploads)
                            .map_err(PomeloError::new)
                    }

                    else {
                        match downloader.search(&query, search_type, page_number).await {
                            Ok(search) => match search_type {
                                SearchType::Video => Ok(SearchResults::Videos(search)),
                                SearchType::Channel => Ok(SearchResults::Channels(search)),
                                SearchType::Playlist => Ok(SearchResults::Playlists(search)),
                                _ => unreachable!()
                            },
                            Err(e) => Err(PomeloError::new(e))
                        }
                    }
                },
                |result| SearchResultsMessage::SearchComplete(result).into()
            ),

            Navigation::None
        )
    }

    // Handle result of search query. Start downloading thumbnails if search was successful.
    fn on_search_complete(&mut self, result: Result<SearchResults, PomeloError>, cache: &PomeloCache) -> (Task<Msg>, Navigation) {
        let command = match &result {
            Ok(search) => {

                info!("Search complete.");

                if let SearchResults::ChannelUploads(videos) = &search {
                    if let Some(cont) = &videos.continuation {
                        self.continuation.insert(self.page_number + 1, cont.clone());
                    }
                }

                super::batch_thumbnail_commands(search, cache)
            },
            Err(e) => {
                error!("Search failed: {}", e.error);
                Task::none()
            }
        };

        self.search_results = Some(result);

        (command, Navigation::None)
    }

    // Navigate to another search results page.
    fn on_new_page(&mut self, page_number: usize) -> (Task<Msg>, Navigation) {

        self.page_number = page_number;
        self.search_results = None;

        (
            Task::done(SearchResultsMessage::StartSearch.into()),
            Navigation::None
        )
    }

    // Generate a scrollable list of search items.
    fn get_search_results_element(&self, search_results: &Result<SearchResults, PomeloError>, instance: &PomeloInstance) -> Element<Msg> {
        use iced::widget::Scrollable;

        let mut column = Column::new().spacing(10);

        match search_results {
            Ok(search) => {
                let mut results = Column::<Msg>::new().spacing(10);
                for item in search.get_results().iter() {
                    let thumbnails = instance.cache().thumbnails();
                    results = results.push(self.get_search_item_element(item, thumbnails));
                }
                column = column.push(
                    Scrollable::new(results)
                        .width(Length::Fill)
                        .height(instance.settings().window_size().1 * 3.0 / 4.0)
                )
            },
            Err(e) => column = column.push(Text::new(e.error.clone()).fill())
        }

        column.into()
    }

    // Generate a button that contains the item's thumbnail and info.
    fn get_search_item_element(&self, item: &SearchResult, thumbnails: &HashMap<String, Handle>) -> Element<Msg> {
        let mut row: Row<Msg> = Row::new();

        if let Some(handle) = thumbnails.get(&item.id()) {
            row = row.push(Image::new(handle.clone()));
        }

        row = row.push(
            Column::from_vec(
                item.info().into_iter()
                    .map(|s| Text::new(s).into())
                    .collect()
            )
        );

        let msg = match item {
            SearchResult::Video(v) => SearchResultsMessage::ToVideo(v.clone()),
            SearchResult::Channel(ch) => SearchResultsMessage::ToChannelVideos(ch.id.clone()),
            SearchResult::Playlist(p) => SearchResultsMessage::ToPlaylistVideos(p.id.clone()),
            _ => unreachable!()
        };

        Button::new(row)
            .width(Length::Fill)
            .on_press(msg.into())
            .into()
    }
}

// Move to video info page with the given video.
fn go_to_video(video: CommonVideo) -> (Task<Msg>, Navigation) {
    use super::video_info_page::VideoInfoPage;

    (
        Task::none(),
        Navigation::GoTo(Box::new(VideoInfoPage::new_with_video(video)))
    )
}

// Move to another search results page that contains this channel's uploaded videos.
fn go_to_channel_videos(id: &str) -> (Task<Msg>, Navigation) {
    (
        Task::done(SearchResultsMessage::StartSearch.into()),
        Navigation::GoTo(
            Box::new(SearchResultsPage::new(String::from(id), SearchType::ChannelUploads))
        )
    )
}

// Move to playlist info page with the given playlist id.
fn go_to_playlist_videos(id: String) -> (Task<Msg>, Navigation) {
    use super::playlist_info_page::{PlaylistInfoMessage, PlaylistInfoPage};

    (
        Task::done(PlaylistInfoMessage::LoadPlaylist(id).into()),
        Navigation::GoTo(
            Box::new(PlaylistInfoPage::new())
        )
    )
}