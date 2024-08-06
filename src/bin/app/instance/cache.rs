use std::collections::HashMap;
use iced::widget::image::Handle;

// Stores items loaded from youtube so that they won't need to be loaded again.
pub (crate) struct PomeloCache {
    // Maps a video, channel, or playlist id to a thumbnail image.
    // The length of each type of id is different, so there shouldn't be any conflicts.
    thumbnails: HashMap<String, Handle>
}

impl PomeloCache {
    pub (crate) fn new() -> Self {
        Self {
            thumbnails: HashMap::new()
        }
    }

    pub (crate) fn thumbnails(&self) -> &HashMap<String, Handle> {
        &self.thumbnails
    }

    pub (crate) fn has_thumbnail(&self, id: &str) -> bool {
        self.thumbnails.contains_key(id)
    }

    pub (crate) fn get_thumbnail(&self, id: &str) -> Option<Handle> {
        self.thumbnails.get(id).cloned()
    }

    pub (crate) fn add_thumbnail(&mut self, id: String, handle: Handle) {
        self.thumbnails.insert(id, handle);
    }
}