use std::fmt::Display;

use iced::Task;
use iced::widget::Text;

use crate::app::PomeloInstance;
use crate::app::instance::settings::INVID_INSTANCES;

use super::{PomeloPage, Navigation, Msg};

// Wrapper for usize, used as an index to the list of Invidious instances.
#[derive(PartialEq, Eq, Clone)]
struct InstanceIndex {
    n: usize
}

impl InstanceIndex {
    fn new(n: usize) -> Self {
        Self { n }
    }
}

impl Display for InstanceIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let instance = INVID_INSTANCES[self.n];
        write!(f, "{} ({})", instance.0, instance.1)
    }
}

#[derive(Debug, Clone)]
pub (crate) enum SettingsMessage {
    InvidiousSetInstance(usize),
    YtUseNightly(bool),
    SetDownloadFolder(String),
    VideoSkipOnError(bool),
    OpenFolderPicker
}

impl From<SettingsMessage> for Msg {
    fn from(value: SettingsMessage) -> Self {
        Msg::Settings(value)
    }
}

// Page that allows users to modify Pomelo settings.
pub (crate) struct SettingsPage;

impl SettingsPage {
    pub (crate) fn new() -> Self {
        Self {}
    }
}

impl PomeloPage for SettingsPage {
    fn update(&mut self, instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation) {

        let settings = instance.settings_mut();

        if let Msg::Back = message {
            (Task::none(), Navigation::Back)
        }

        else if let Msg::Settings(msg) = message {
            match msg {
                SettingsMessage::InvidiousSetInstance(index) 
                    => settings.set_invidious_index(index),
        
                SettingsMessage::YtUseNightly(checked) 
                    => settings.set_use_nightly(checked),

                SettingsMessage::SetDownloadFolder(path) 
                    => settings.set_download_folder(&path),

                SettingsMessage::VideoSkipOnError(checked) 
                    => settings.set_video_skip_on_error(checked),

                SettingsMessage::OpenFolderPicker => return (
                    open_folder_picker(instance.settings().download_folder()),
                    Navigation::None
                )
            }

            (Task::none(), Navigation::None)
        }

        else {
            (Task::none(), Navigation::None)
        }
    }

    fn view(&self, instance: &PomeloInstance) -> iced::Element<Msg> {
        use iced::widget::{column, row, PickList, Button, Checkbox, TextInput};
        use super::FillElement;

        column![

            // Invidious options
            column![
                header("Invidious"),

                row![
                    tooltip_with_background(
                        "Instance",
                        "3rd-party Youtube API used for searching.\n\
                        Try changing this if searching doesn't work."
                    ),

                    PickList::new(
                        (0..INVID_INSTANCES.len())
                            .map(InstanceIndex::new)
                            .collect::<Vec<_>>(),
                        Some(InstanceIndex::new(instance.settings().invidious_index())),
                        |index| SettingsMessage::InvidiousSetInstance(index.n).into()
                    )
                ].spacing(10)
            ].spacing(10).align_x(iced::Alignment::Center),

            // Yt-dlp options
            column![
                header("Yt-dlp"),

                row![
                    tooltip_with_background(
                        "Use nightly build",
                        "Use the latest nightly release of yt-dlp, instead of the stable one.\n\
                        Try changing this if downloads don't work or stop working."
                    ),

                    Checkbox::new("", instance.settings().use_nightly())
                        .on_toggle(|checked| SettingsMessage::YtUseNightly(checked).into())
                ].spacing(10),

                row![
                    Text::new("Download Folder"),
                    TextInput::new("", instance.settings().download_folder()).width(350),
                    Button::new(Text::new("Change").center())
                        .width(100)
                        .on_press(SettingsMessage::OpenFolderPicker.into())
                ].spacing(10)
            ].spacing(10).align_x(iced::Alignment::Center),

            // Video options
            column![
                header("Playback"),

                row![
                    Text::new("Auto-skip on error"),

                    Checkbox::new("", instance.settings().video_skip_on_error())
                        .on_toggle(|checked| SettingsMessage::VideoSkipOnError(checked).into()),

                ].spacing(10)
            ].spacing(10).align_x(iced::Alignment::Center),

            Button::new(Text::new("Back").center())
                .width(100)
                .on_press(Msg::Back)

        ].spacing(25).align_x(iced::Alignment::Center).fill()
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<Msg> {
        iced::Subscription::none()
    }
}

fn header(text: &str) -> iced::Element<Msg> {
    use iced::font::{Font, Weight};

    Text::new(text).font(
        Font {
            weight: Weight::Bold,
            ..Default::default()
        }
    ).size(24).into()
}

fn tooltip_with_background <'a> (text: &'a str, tip: &'a str) -> iced::Element<'a, Msg> {
    use iced::widget::{Container, Tooltip};
    use iced::widget::container;
    use iced::widget::tooltip::Position;

    Tooltip::new(
        Text::new(text),
        Container::new(Text::new(tip)).style(
            |e: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(e.palette().primary)),
                border: iced::Border {
                    color: iced::Color::BLACK,
                    width: 2.5,
                    radius: iced::border::Radius::new(10)
                },
                ..Default::default()
            }
        ).padding(10),
        Position::default()
    ).into()
}

fn open_folder_picker(path: &str) -> Task<Msg> {
    use rfd::FileDialog;

    let maybe_folder = FileDialog::new()
        .set_directory(path)
        .pick_folder();

    if let Some(folder) = maybe_folder {
        Task::done(
            SettingsMessage::SetDownloadFolder(folder.to_str().unwrap().replace('\\', "/")).into()
        )
    }
    else {
        Task::none()
    }
}