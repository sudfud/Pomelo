use iced::Task;

use crate::yt_fetch::SearchType;

use super::{PomeloInstance, PomeloPage, Navigation, Msg};

#[derive(Debug, Clone)]
pub (crate) enum SearchMessage {
    UpdateInput(String),
    SetSearchType(SearchType),
    SubmitQuery
}

impl From<SearchMessage> for Msg {
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
    fn update(&mut self, _instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation) {
        if let Msg::Back = message {
            return (Task::none(), Navigation::Back);
        }

        else if let Msg::Search(msg) = message {
            match msg {
                SearchMessage::UpdateInput(s) => self.search_input = s,
                SearchMessage::SetSearchType(s_type) => self.search_type = s_type,
                SearchMessage::SubmitQuery => return self.submit_query()
            }
        }

        (Task::none(), Navigation::None)
    }

    fn view(&self, instance: &PomeloInstance) -> iced::Element<Msg> {
        use iced::Length;
        use iced::widget::{column, row, TextInput, Radio};
        use super::{centered_text_button, FillElement};

        let input = TextInput::new("Search or Enter Youtube URL", &self.search_input)
            .on_input(|s| SearchMessage::UpdateInput(s).into())
            .on_submit(SearchMessage::SubmitQuery.into())
            .padding(10)
            .width(instance.settings().window_size().0 / 2.0);

        let set_search_type = |s_type| SearchMessage::SetSearchType(s_type).into();


        column![
            input,
            row![
                Radio::<Msg>::new(
                    "Videos",
                    SearchType::Video,
                    Some(self.search_type),
                    set_search_type
                ),
                Radio::<Msg>::new(
                    "Channels",
                    SearchType::Channel,
                    Some(self.search_type),
                    set_search_type
                ),
                Radio::<Msg>::new(
                    "Playlists",
                    SearchType::Playlist,
                    Some(self.search_type),
                    set_search_type
                )
            ].spacing(10),

            centered_text_button("Search", Some(100), None::<Length>)
                .on_press(SearchMessage::SubmitQuery.into()),

            centered_text_button("Back", Some(100), None::<Length>).on_press(Msg::Back)

        ].spacing(25).align_x(iced::Alignment::Center).fill()

    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<Msg> {
        iced::Subscription::none()
    }
}

impl SearchPage {
    
    // Move to video info page if query is a URL, otherwise move to search results page with query.
    fn submit_query(&self) -> (Task<Msg>, Navigation) {
        use super::video_info_page::{VideoInfoMessage, VideoInfoPage};
        use super::search_results_page::{SearchResultsMessage, SearchResultsPage};

        if self.search_input.starts_with("https://") {
            let query = rusty_ytdl::get_video_id(&self.search_input).unwrap();

            (
                Task::done(VideoInfoMessage::LoadVideo(query).into()),
                Navigation::GoTo(Box::new(VideoInfoPage::new()))
            )
        }

        else {
            let query = self.search_input.clone();
            let s_type = self.search_type;

            (
                Task::done(SearchResultsMessage::StartSearch.into()),
                Navigation::GoTo(Box::new(SearchResultsPage::new(query, s_type)))
            )
        }
    }
}