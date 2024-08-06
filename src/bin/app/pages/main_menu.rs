use iced::Task;

use super::{Navigation, PomeloPage, PomeloInstance, Msg};

// Main menu, the first page that's loaded when the program starts.
// Redirects to the Settings, Search, and Video Player pages.
pub (crate) struct MainMenu;

#[derive(Debug, Clone)]
pub (crate) enum MainMenuMessage {
    LocalVideoPage,
    SearchPage,
    SettingsPage
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
                MainMenuMessage::LocalVideoPage => return to_local_video_page(),
                MainMenuMessage::SearchPage => return (
                    Task::none(),
                    Navigation::GoTo(Box::new(SearchPage::new()))
                ),
                MainMenuMessage::SettingsPage => return (
                    Task::none(),
                    Navigation::GoTo(Box::new(SettingsPage {}))
                )
            }
        }
        (Task::none(), Navigation::None)
    }

    fn view(&self, _instance: &PomeloInstance) -> iced::Element<Msg> {
        use iced::Length;
        use super::{centered_text_button, FillElement};

        // Draw buttons
        iced::widget::column![
            centered_text_button("Play from Computer", Some(200), None::<Length>)
                .on_press(MainMenuMessage::LocalVideoPage.into()),
            centered_text_button("Play from Youtube", Some(200), None::<Length>)
                .on_press(MainMenuMessage::SearchPage.into()),
            centered_text_button("Settings", Some(200), None::<Length>)
                .on_press(MainMenuMessage::SettingsPage.into())
        ].spacing(25).fill()
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<Msg> {
        iced::Subscription::none()
    }
}

fn to_local_video_page() -> (Task<Msg>, Navigation) {
    use super::local_video_page::LocalVideoPage;

    (
        Task::none(),
        Navigation::GoTo(Box::new(LocalVideoPage::new()))
    )
}