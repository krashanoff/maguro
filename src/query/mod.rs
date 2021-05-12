//! Query resolved by maguro.
//!
//! Handles parsing channel, video, playlist URLs and IDs into maguro-managed
//! entities.

use std::{error, str::FromStr};

use lazy_static::lazy_static;
use regex::Regex;

/// Collection of video IDs that will be downloaded, as parsed from
/// a list of YouTube channels, playlists, video URLs.
pub struct Query(String);

impl Query {
    /// Video URLs parsed from a given query.
    pub async fn urls(&self) -> Result<Vec<String>, Box<dyn error::Error>> {
        lazy_static! {
          // First capture group is always our video ID.
          static ref VIDEO: Regex = Regex::new("").unwrap();
        }

        let mut videos = Vec::new();
        for pattern in self.0.split(" ") {
            videos.push(format!(
                "https://www.youtube.com/get_video_info?video_id={}",
                pattern
            ));
        }
        Ok(videos)
    }
}

impl FromStr for Query {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn simple_ids() {
        if let Err(_) = Query::from_str("VfWgE7D1pYY").unwrap().urls().await {
            assert!(false);
        }
    }
}
