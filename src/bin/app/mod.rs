mod pages;
mod instance;

use iced::window;
use iced::{Size, Task};

use log::warn;

use instance::PomeloInstance;
use instance::settings::PomeloSettings;

// Youtube thumbnails, represented as a 2-tuple with the youtube id (String) and the image data (Handle).
type Thumbnail = (String, iced::widget::image::Handle);

// Simple wrapper for errors.
#[derive(Debug, Clone)]
pub (crate) struct PomeloError {
    error: String
}

impl PomeloError {
    fn new(e: impl std::error::Error + 'static) -> Self {
        Self { error: e.to_string() }
    }
}

impl From<String> for PomeloError {
    fn from(value: String) -> Self {
        Self { error: value }
    }
}

impl From<&str> for PomeloError {
    fn from(value: &str) -> Self {
        Self { error: String::from(value) }
    }
}

// Messages are used to update the state of the program.
#[derive(Debug, Clone)]
pub (crate) enum PomeloMessage {
    Init,

    MainMenu(pages::MainMenuMessage),
    LocalVideo(pages::LocalVideoMessage),
    VideoPlayer(pages::VideoPlayerMessage),
    Search(pages::SearchMessage),
    SearchResults(pages::SearchResultsMessage),
    VideoInfo(pages::VideoInfoMessage),
    PlaylistInfo(pages::PlaylistInfoMessage),
    
    WindowResize((window::Id, Size)),
    InvidiousSetInstance(usize),
    YtUseNightly(bool),
    VideoSkipOnError(bool),

    ThumbnailLoaded(Result<Thumbnail, PomeloError>),

    Back,
    //Home,

    Close(window::Id)
}

// The "heart" of Pomelo.
pub (crate) struct PomeloApp {
    instance: PomeloInstance,
    page_stack: Vec<Box<dyn pages::PomeloPage>>
}

impl PomeloApp {
    pub (crate) fn new() -> (Self, Task<PomeloMessage>) {
        use iced::advanced::graphics::image::image_rs::ImageFormat;
        
        
        let settings = match PomeloSettings::load() {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to load settings, using defaults: {}", e.error);
                PomeloSettings::new()
            }
        };

        let window_settings = iced::window::Settings {
            size: Size::from(settings.window_size()),
            min_size: Some(Size::new(500.0, 500.0)),
            icon: window::icon::from_file_data(include_bytes!("../../../icon.png"), Some(ImageFormat::Png))
                .ok(),
            exit_on_close_request: true,
            ..Default::default()
        };

        let (_, window) = window::open(window_settings);

        let app = PomeloApp {
            instance: PomeloInstance::new(settings),
            page_stack: vec![Box::new(pages::MainMenu {})]
        };

        (app, window.map(|_| PomeloMessage::Init))
    }

    // Sets the title of the program window.
    pub (crate) fn title(&self, _id: window::Id) -> String {
        String::from("Pomelo")
    }

    // Update the state of the program.
    pub (crate) fn update(&mut self, message: PomeloMessage) -> Task<PomeloMessage> {
        use pages::Navigation;

        match message {
            PomeloMessage::WindowResize((_id, size)) => {
                self.instance.settings_mut().set_window_size(size.width, size.height);
                Task::none()
            },

            PomeloMessage::InvidiousSetInstance(index) => {
                self.instance.settings_mut().set_invidious_index(index);
                Task::none()
            },
    
            PomeloMessage::YtUseNightly(checked) => {
                self.instance.settings_mut().set_use_nightly(checked);
                Task::none()
            },

            PomeloMessage::VideoSkipOnError(checked) => {
                self.instance.settings_mut().set_video_skip_on_error(checked);
                Task::none()            
            }
    
            PomeloMessage::ThumbnailLoaded(result) => {
                if let Ok((id, handle)) = result {
                    self.instance.cache_mut().add_thumbnail(id, handle);
                }
                Task::none()
            },

            PomeloMessage::Close(_id) => {
                self.instance.cancel_download();
                self.instance.settings().save();

                iced::exit()
            },

            // Retrieve command(s) and navigation info from the current page
            _ => {
                let current_page = self.page_stack
                    .last_mut()
                    .expect("Page stack should not be empty.");

                let (command, navigation) = current_page.update(&mut self.instance, message);

                match navigation {
                    Navigation::GoTo(page) => self.page_stack.push(page),
                    Navigation::Back => {self.page_stack.pop();},
                    Navigation::None => {}
                }

                command
            }
        }
    }

    // Draw the current page's UI.
    pub (crate) fn view(&self, _id: window::Id) -> iced::Element<PomeloMessage> {
        self.page_stack.last().unwrap().view(&self.instance)
    }

    // Handle user input.
    pub (crate) fn subscription(&self) -> iced::Subscription<PomeloMessage> {
        iced::Subscription::batch(
            [
                window::resize_events().map(PomeloMessage::WindowResize),
                window::close_events().map(PomeloMessage::Close),
                self.page_stack.last().unwrap().subscription(&self.instance)
            ]
        )
    }
}