use iced::Task;

use crate::app::{PomeloCommand, PomeloMessage};

use super::local_video_page::LocalVideoPage;
use super::{PomeloPage, PomeloInstance};

// Main menu, the first page that's loaded when the program starts.
// Redirects to the Settings, Search, and Video Player pages.
pub (crate) struct MainMenu;

#[derive(Debug, Clone)]
pub (crate) enum MainMenuMessage {
    LocalVideo,
    Search,
    Settings
}

impl From<MainMenuMessage> for PomeloMessage {
    fn from(value: MainMenuMessage) -> Self {
        Self::MainMenu(value)
    }
}

impl PomeloPage for MainMenu {
    
    fn update(&mut self, _instance: &mut PomeloInstance, message: PomeloMessage) -> PomeloCommand {
        use super::search_page::SearchPage;
        use super::settings_page::SettingsPage;

        match message {
            PomeloMessage::MainMenu(msg) => {
                match msg {
                    MainMenuMessage::LocalVideo => PomeloCommand::go_to(LocalVideoPage::new()),
                    MainMenuMessage::Search => PomeloCommand::go_to(SearchPage::new()),
                    MainMenuMessage::Settings => PomeloCommand::go_to(SettingsPage::new())
                }
            },

            _ => PomeloCommand::none()
        }
    }

    fn view(&self, _instance: &PomeloInstance) -> iced::Element<PomeloMessage> {
        use super::{FillElement, simple_button};

        // Draw buttons
        iced::widget::column![
            simple_button("Play from Computer", 200, MainMenuMessage::LocalVideo),
            simple_button("Play from Youtube", 200, MainMenuMessage::Search),
            simple_button("Settings", 200, MainMenuMessage::Settings)
        ].spacing(25).fill()
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<PomeloMessage> {
        iced::Subscription::none()
    }
}