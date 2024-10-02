use iced::Task;

use crate::app::{PomeloMessage, PomeloCommand};
use super::{PomeloInstance, PomeloPage, Navigation, yt_fetch::SearchType};

#[derive(Debug, Clone)]
pub (crate) enum SearchMessage {
    UpdateInput(String),
    SetSearchType(SearchType),
    SubmitQuery
}

impl From<SearchMessage> for PomeloMessage {
    fn from(value: SearchMessage) -> Self {
        Self::Search(value)
    }
}

// Page for search queries. Can be used to play videos directly, or to search for videos/channels/playlists from Youtube.
pub (crate) struct SearchPage {
    search_input: String,
    search_type: SearchType,
}

impl SearchPage {
    pub (crate) fn new() -> Self {
        Self {
            search_input: String::new(),
            search_type: SearchType::Video
        }
    }
}

impl PomeloPage for SearchPage {
    fn update(&mut self, _instance: &mut PomeloInstance, message: PomeloMessage) -> PomeloCommand {
        if let PomeloMessage::Back = message {
            return PomeloCommand::back();
        }

        else if let PomeloMessage::Search(msg) = message {
            match msg {
                SearchMessage::UpdateInput(s) => self.search_input = s,
                SearchMessage::SetSearchType(s_type) => self.search_type = s_type,
                SearchMessage::SubmitQuery => return self.submit_query()
            }
        }

        PomeloCommand::none()
    }

    fn view(&self, instance: &PomeloInstance) -> iced::Element<PomeloMessage> {
        use iced::widget::{column, row, TextInput, Radio, Button, Text};
        use super::FillElement;

        let input = TextInput::new("Search or Enter Youtube URL", &self.search_input)
            .on_input(|s| SearchMessage::UpdateInput(s).into())
            .on_submit(SearchMessage::SubmitQuery.into())
            .padding(10)
            .width(instance.settings().window_size().0 / 2.0);

        let set_search_type = |s_type| SearchMessage::SetSearchType(s_type).into();

        column![
            input,
            row![
                Radio::<PomeloMessage>::new(
                    "Videos",
                    SearchType::Video,
                    Some(self.search_type),
                    set_search_type
                ),
                Radio::<PomeloMessage>::new(
                    "Channels",
                    SearchType::Channel,
                    Some(self.search_type),
                    set_search_type
                ),
                Radio::<PomeloMessage>::new(
                    "Playlists",
                    SearchType::Playlist,
                    Some(self.search_type),
                    set_search_type
                )
            ].spacing(10),

            Button::new(Text::new("Search").center())
                .width(100)
                .on_press(SearchMessage::SubmitQuery.into()),

            Button::new(Text::new("Back").center())
                .width(100)
                .on_press(PomeloMessage::Back)

        ].spacing(25).align_x(iced::Alignment::Center).fill()
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<PomeloMessage> {
        iced::Subscription::none()
    }
}

impl SearchPage {
    
    // Move to video info page if query is a URL, otherwise move to search results page with query.
    fn submit_query(&self) -> PomeloCommand {
        use super::video_info_page::{VideoInfoMessage, VideoInfoPage};
        use super::search_results_page::{SearchResultsMessage, SearchResultsPage};

        if self.search_input.starts_with("https://") {
            let query = rusty_ytdl::get_video_id(&self.search_input).unwrap();

            PomeloCommand::go_to_with_message(VideoInfoMessage::LoadVideo(query), VideoInfoPage::new())
        }

        else {
            let query = self.search_input.clone();
            let s_type = self.search_type;

            PomeloCommand::new(
                Task::done(SearchResultsMessage::StartSearch.into()),
                Navigation::GoTo(Box::new(SearchResultsPage::new(query, s_type)))
            )
        }
    }
}