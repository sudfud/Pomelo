use std::fmt::Display;

use iced::Task;

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
        use iced::widget::{column, row, Text, PickList, Button, Checkbox, Tooltip, Container};
        use iced::widget::tooltip::Position;
        use iced::widget::container;
        use iced::font::{Font, Weight};
        use super::FillElement;

        column![

            // Invidious options
            column![
                Text::new("Invidious").font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }).size(24),
                row![
                    Tooltip::new(
                        Text::new("Instance"),
                        Container::new(
                            Text::new(
                                "3rd-party Youtube API used for searching.\n\
                                Try changing this if searching doesn't work."
                            )
                        ).style(
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
                Text::new("Yt-dlp").font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }).size(24),
                row![
                    Tooltip::new(
                        Text::new("Use nightly build"),
                        Container::new(
                            Text::new(
                                "Use the latest nightly release of yt-dlp, instead of the stable one.\n\
                                Try changing this if downloads don't work or stop working."
                            )
                        ).style(
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
                    ),

                    Checkbox::new("", instance.settings().use_nightly())
                        .on_toggle(Msg::YtUseNightly)
                ].spacing(5)
            ].align_x(iced::Alignment::Center),

            Button::new("Back").on_press(Msg::Back)
        ].spacing(25).align_x(iced::Alignment::Center).fill()
    }

    fn subscription(&self, _instance: &PomeloInstance) -> iced::Subscription<Msg> {
        iced::Subscription::none()
    }
}