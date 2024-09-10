use iced::Task;

use crate::app::pages::local_video_page::LocalVideoPage;

use super::{Navigation, PomeloPage, PomeloInstance, Msg};

// Main menu, the first page that's loaded when the program starts.
// Redirects to the Settings, Search, and Video Player pages.
pub (crate) struct MainMenu;

#[derive(Debug, Clone)]
pub (crate) enum MainMenuMessage {
    LocalVideo,
    Search,
    Settings
}

impl From<MainMenuMessage> for Msg {
    fn from(value: MainMenuMessage) -> Self {
        Self::MainMenu(value)
    }
}

impl PomeloPage for MainMenu {
    
    fn update(&mut self, _instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation) {
        use super::search_page::SearchPage;
        use super::settings_page::SettingsPage;

        if let Msg::MainMenu(msg) = message {
            match msg {
                MainMenuMessage::LocalVideo => return go_to_page(LocalVideoPage::new()),
                MainMenuMessage::Search => return go_to_page(SearchPage::new()),
                MainMenuMessage::Settings => return go_to_page(SettingsPage::new())
            }
        }
        (Task::none(), Navigation::None)
    }

    fn view(&self, _instance: &PomeloInstance) -> iced::Element<Msg> {
        use iced::widget::{Button, Text};
        use super::FillElement;

        // Draw buttons
        iced::widget::column![
            Button::new(Text::new("Play from Computer").center())
                .width(200)
                .on_press(MainMenuMessage::LocalVideo.into()),

            Button::new(Text::new("Play from Youtube").center())
                .width(200)
                .on_press(MainMenuMessage::Search.into()),

            Button::new(Text::new("Settings").center())
                .width(200)
                .on_press(MainMenuMessage::Settings.into())
        ].spacing(25).fill()
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<Msg> {
        iced::Subscription::none()
    }
}

fn go_to_page(page: impl PomeloPage + 'static) -> (Task<Msg>, Navigation) {
    (Task::none(), Navigation::GoTo(Box::new(page)))
}