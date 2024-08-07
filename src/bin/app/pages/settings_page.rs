use std::fmt::Display;

use iced::Task;
use iced::widget::Text;

use crate::INVID_INSTANCES;
use crate::app::PomeloInstance;

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

// Page that allows users to modify Pomelo settings.
pub (crate) struct SettingsPage;

impl PomeloPage for SettingsPage {
    fn update(&mut self, _instance: &mut PomeloInstance, message: Msg) -> (Task<Msg>, Navigation) {
        if let Msg::Back = message {
            return (Task::none(), Navigation::Back);
        }

        (Task::none(), Navigation::None)
    }

    fn view(&self, instance: &PomeloInstance) -> iced::Element<Msg> {
        use iced::widget::{column, row, PickList, Button, Checkbox};
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
                        |index| Msg::InvidiousSetInstance(index.n)
                    )
                ].spacing(10)
            ].align_x(iced::Alignment::Center),

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
                        .on_toggle(Msg::YtUseNightly)
                ].spacing(5)
            ].align_x(iced::Alignment::Center),

            // Video options
            column![
                header("Playback"),

                row![
                    Text::new("Auto-skip on error"),

                    Checkbox::new("", instance.settings().video_skip_on_error())
                        .on_toggle(Msg::VideoSkipOnError)
                ].spacing(5)
            ],

            Button::new("Back").on_press(Msg::Back)
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