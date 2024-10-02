use std::collections::HashMap;

use iced::{Task, Length, Element};
use iced::widget::{column, row, Column, Row, Text, Button, Image};
use iced::widget::image::Handle;
use invidious::CommonVideo;
use log::{info, error};

use crate::app::{PomeloError, PomeloMessage, PomeloCommand};
use crate::app::instance::cache::PomeloCache;

use super::{FillElement, PomeloInstance, Navigation};
use super::yt_fetch::{SearchResult, SearchResults, SearchType, VideoFetcher};

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
                format!("{} Subscribers", ch.subscribers)
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

impl From<SearchResultsMessage> for PomeloMessage {
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
    fn update(&mut self, instance: &mut PomeloInstance, message: PomeloMessage) -> PomeloCommand {
        use super::video_info_page::VideoInfoPage;
        use super::playlist_info_page::{PlaylistInfoMessage, PlaylistInfoPage};

        match message {
            PomeloMessage::Back => PomeloCommand::back(),
            PomeloMessage::Home => PomeloCommand::home(),
            PomeloMessage::SearchResults(msg) => match msg {
                SearchResultsMessage::StartSearch 
                    => self.start_search(instance.settings().invidious_url()),

                SearchResultsMessage::SearchComplete(result) 
                    => self.on_search_complete(result, instance.cache()),

                SearchResultsMessage::NewPage(page_number) 
                    => self.on_new_page(page_number),

                SearchResultsMessage::ToVideo(video) 
                    => PomeloCommand::go_to(VideoInfoPage::new_with_video(video)),

                SearchResultsMessage::ToChannelVideos(id) 
                    => PomeloCommand::go_to_with_message(SearchResultsMessage::StartSearch, SearchResultsPage::new(id, SearchType::ChannelUploads)),

                SearchResultsMessage::ToPlaylistVideos(id)
                    => PomeloCommand::go_to_with_message(PlaylistInfoMessage::LoadPlaylist(id), PlaylistInfoPage::new())
            },
            _ => PomeloCommand::none()
        }
    }

    fn view(&self, instance: &PomeloInstance) -> Element<PomeloMessage> {
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
                    .on_press(PomeloMessage::Back),
            
                Button::new(Text::new("Next").center())
                    .width(100)
                    .on_press(SearchResultsMessage::NewPage(self.page_number+1).into())
            
            ].spacing(25);
    
            column![
                result_element,
                buttons,
                Button::new(Text::new("Home").center())
                    .width(100)
                    .on_press(PomeloMessage::Home)
            ].align_x(iced::Alignment::Center).spacing(25).into()
        }
        else {
            "Loading...".fill()
        }
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<PomeloMessage> {
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
    fn start_search(&self, invid_url: &str) -> PomeloCommand {
        let query = self.query.clone();
        let search_type = self.search_type;
        let page_number = self.page_number;
        let continuation = self.continuation.get(&self.page_number).cloned();

        info!("Starting Youtube search. Type: {}, Page: {}, Query: {}", search_type, page_number, query);
        let downloader = VideoFetcher::new(invid_url);

        PomeloCommand::new(
            Task::perform(
                async move {

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
    fn on_search_complete(&mut self, result: Result<SearchResults, PomeloError>, cache: &PomeloCache) -> PomeloCommand {
        let task = match &result {
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

        PomeloCommand::task_only(task)
    }

    // Navigate to another search results page.
    fn on_new_page(&mut self, page_number: usize) -> PomeloCommand {

        self.page_number = page_number;
        self.search_results = None;

        
        PomeloCommand::message(SearchResultsMessage::StartSearch)
    }

    // Generate a scrollable list of search items.
    fn get_search_results_element(&self, search_results: &Result<SearchResults, PomeloError>, instance: &PomeloInstance) -> Element<PomeloMessage> {
        use iced::widget::Scrollable;

        let mut column = Column::new().spacing(10);

        match search_results {
            Ok(search) => {
                let mut results = Column::<PomeloMessage>::new().spacing(10);
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
    fn get_search_item_element(&self, item: &SearchResult, thumbnails: &HashMap<String, Handle>) -> Element<PomeloMessage> {
        let mut row: Row<PomeloMessage> = Row::new();

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