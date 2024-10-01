/*
 * The yt_fetch module contains functionality for retrieving videos and video information from Youtube.
 * 
 * The Invidious API is used to get video information ( viewcount, thumbnail, etc. ).
 * For some reason, Invidious can't be used to get the actual videos themselves, so the rusty_ytdl crate serves this purpose instead.
 */

use std::time::Duration;

use iced::widget::image::Handle;

use invidious::{
    channel::ChannelVideos,
    hidden::{PlaylistItem, SearchItem},
    universal::{Playlist, Search},
    video::Video as VideoDetails,
    ClientAsync,
    ClientAsyncTrait,
    CommonChannel,
    CommonPlaylist,
    CommonVideo,
    InvidiousError,
    MethodAsync
};

// Wrapper for various types errors that can occur.
#[derive(Debug)]
pub (crate) struct FetchError {
    error: String
}

impl FetchError {
    fn new(error: String) -> Self {
        Self { error }
    }
}

impl From<InvidiousError> for FetchError {
    fn from(value: InvidiousError) -> Self {
        Self::new(value.to_string())
    }
}

impl From<reqwest::Error> for FetchError {
    fn from(value: reqwest::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for FetchError {
    fn from(value: tokio::time::error::Elapsed) -> Self {
        Self::new(value.to_string())
    }
}

impl From<&str> for FetchError {
    fn from(value: &str) -> Self {
        Self::new(String::from(value))
    }
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for FetchError {}

// We use our own SearchType enum instead of rusty_ytdl's
// rusty's SearchType doesn't implement Copy or Eq, which are needed for the radio buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    Video,
    Channel,
    ChannelUploads,
    Playlist,
}

impl std::fmt::Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SearchType::Video => "Video",
            SearchType::Channel => "Channel",
            SearchType::ChannelUploads => "ChannelUploads",
            SearchType::Playlist => "Playlist"
        };

        write!(f, "{}", s)
    }
}

// Wrapper for search result items. 
#[derive(Debug, Clone)]
pub enum SearchResult {
    Video(CommonVideo),
    Channel(CommonChannel),
    Playlist(CommonPlaylist),
    PlaylistVideo(PlaylistItem)
}

impl From<SearchItem> for SearchResult {
    fn from(value: SearchItem) -> Self {
        match value {
            SearchItem::Video(video) => SearchResult::Video(video),
            SearchItem::Channel(channel) => SearchResult::Channel(channel),
            SearchItem::Playlist(playlist) => SearchResult::Playlist(playlist)
        }
    }
}

impl From<PlaylistItem> for SearchResult {
    fn from(value: PlaylistItem) -> Self {
        SearchResult::PlaylistVideo(value)
    }
}

// Wraps different search results to a single enum.
#[derive(Debug, Clone)]
pub enum SearchResults {
    Videos(Search),
    Channels(Search),
    ChannelUploads(ChannelVideos),
    Playlists(Search),
    PlaylistVideos(Playlist)
}

impl SearchResults {
    pub fn get_results(&self) -> Vec<SearchResult> {
        match self {
            SearchResults::Videos(search) |
            SearchResults::Channels(search) |
            SearchResults::Playlists(search) => search.items.iter()
                .map(|item| item.clone().into())
                .collect(),

            SearchResults::ChannelUploads(ch) => ch.videos.iter()
                .map(|video| SearchItem::Video(video.clone()).into())
                .collect(),

            SearchResults::PlaylistVideos(playlist) => playlist.videos.iter()
                .map(|video| video.clone().into())
                .collect()
        }
    }
}

// Wrapper for Invidious that can perform searches and extract information from Youtube.
pub struct VideoFetcher {
    client: ClientAsync
}

impl VideoFetcher {
    pub fn new(instance: impl Into<String>) -> Self {
        Self { client: ClientAsync::new(instance.into(), MethodAsync::Reqwest) }
    }

    pub fn set_instance(&mut self, instance: &str) {
        self.client.set_instance(String::from(instance));
    }

    // Get information about a Youtube video with the given id.
    pub async fn get_video_details(&self, id: &str) -> Result<VideoDetails, FetchError> {
        self.client.video(id, None).await.map_err(FetchError::from)
    }

    // Performs a Youtube search. Times out after 10 seconds.
    pub async fn search(&self, query: &str, search_type: SearchType, page: usize) -> Result<Search, FetchError> {
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            self.client.search(
                Some(&format!("q={}&type={}&page={}",
                urlencoding::encode(query), search_type, page))
            )
        ).await;

        match result {
            Ok(out) => out.map_err(FetchError::from),
            Err(e) => Err(e.into())
        }
    }

    // Get a list of videos from a channel with the given id, continuation determines which page of videos to return.
    // Times out after 10 seconds.
    pub async fn get_channel_videos(&self, channel_id: &str, continuation: Option<&str>) -> Result<ChannelVideos, FetchError> {
        let params = continuation
            .map(|c| format!("continuation={}", c));

        let result = tokio::time::timeout(
            Duration::from_secs(10),
            self.client.channel_videos(channel_id, params.as_deref())
        ).await;

        match result {
            Ok(out) => out.map_err(FetchError::from),
            Err(e) => Err(e.into())
        }
    }

    // Get a list of playlist videos from Youtube with a given id. Times out after 10 seconds.
    pub async fn get_playlist_videos(&self, id: &str) -> Result<Playlist, FetchError> {
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            self.client.playlist(id, None)
        ).await;

        match result {
            Ok(out) => out.map_err(FetchError::from),
            Err(e) => Err(e.into())
        }
    }
}

// Grab a video, channel, playlist thumbnail from Youtube.
pub (crate) async fn download_thumbnail(item: &SearchResult, index: usize) -> Result<Handle, FetchError> {
    match item {
        SearchResult::Video(v) => match v.thumbnails.get(index) {
            Some(thumbnail) => match reqwest::get(&thumbnail.url).await {
                Ok(response) => Ok(Handle::from_bytes(response.bytes().await.unwrap())),
                Err(e) => Err(FetchError::from(e))
            },
            None => Err(FetchError::new(format!("Thumbnail index {} is invalid.", index)))
        },

        SearchResult::Channel(ch) => match ch.thumbnails.get(index) {
            Some(thumbnail) => match reqwest::get(format!("https:{}", &thumbnail.url)).await {
                Ok(response) => Ok(Handle::from_bytes(response.bytes().await.unwrap())),
                Err(e) => Err(FetchError::from(e))
            },
            None => Err(FetchError::new(format!("Thumbnail index {} is invalid.", index)))
        },

        SearchResult::Playlist(playlist) => match reqwest::get(&playlist.thumbnail).await {
            Ok(response) => Ok(Handle::from_bytes(response.bytes().await.unwrap())),
            Err(e) => Err(FetchError::from(e))
        },

        SearchResult::PlaylistVideo(video) => match video.thumbnails.get(index) {
            Some(thumbnail) => {
                match reqwest::get(&thumbnail.url).await {
                    Ok(response) => {
                        Ok(Handle::from_bytes(response.bytes().await.unwrap()))
                    },
                    Err(e) => {
                        Err(FetchError::from(e))
                    }
                }
            },
            None => Err(FetchError::new(format!("Thumbnail index {} is invalid.", index)))
        }
    }
}