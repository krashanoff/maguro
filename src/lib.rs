//! # maguro
//!
//! An async library for downloading and streaming media, with
//! out-of-the-box support for YouTube.
//!
//! ## Features
//!
//! maguro comes out of the box with functions for downloading DASH-MPEG
//! manifests.
//!
//! By default, maguro's default implementations built on [hyper](https://hyper.rs/)
//! are used. To disable them, set your features to anything else.
//!
//! If you would like to implement your own downloader functions, change
//! your features to `["custom"]`.
//!
//! ## Example
//!
//! ```
//! use maguro;
//! use tokio::fs::OpenOptions;
//!
//! // ...
//!
//! // Get our video information and location the first format
//! // available.
//! let video_info = maguro::get_video_info("VfWgE7D1pYY").await?;
//! let format = video_info.all_formats().first().cloned()?;
//!
//! // Open an asynchronous file handle.
//! let mut output = OpenOptions::new()
//!     .read(false)
//!     .write(true)
//!     .create(true)
//!     .open("maguro.mp4")
//!     .await?;
//!
//! // Download the video.
//! format.download(&mut output).await?;
//! ```

use async_trait::async_trait;
use std::{
    fmt::{self, Display},
    str::FromStr,
};

pub mod dash;
mod serde;

#[derive(Debug, Clone)]
/// Error returned from a [Downloader] operation.
pub struct Error(Option<String>);

impl Error {
    pub fn custom<T: Display>(e: T) -> Self {
        Self(Some(e.to_string()))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "custom: {}",
            match &self.0 {
                Some(msg) => msg.as_str(),
                None => "unknown querying error",
            }
        )
    }
}

#[async_trait]
/// Conversion from [String] to a [Query] against a [Downloader]. Type signatures
/// may be imposing, but this is an [async_trait].
pub trait Query: FromStr + Send + Sync {
    /// Type returned as result of a successful query. Must be [String] convertible.
    type URL: ToString + Send;

    type Error: Display + Send;

    #[cfg(feature = "client")]
    /// Parse an input [String] into a vector of URLs. In the event that
    /// a vector of URLs can only be formed *in reference to* some live asset.
    async fn to_urls(&self) -> Result<Vec<Self::URL>, Self::Error>;
}

#[async_trait]
/// A type that is capable of downloading a video via maguro. Type signatures
/// may be imposing, but this is an [async_trait].
pub trait Downloader {
    /// Bound on what is accepted as a query for this [Downloader]. Queries
    /// **must** be parsable from a [&str], and convertible (once parsed)
    /// to a set of URLs pointing to DASH-MPEG Manifests, as described in
    /// [crate::dash]. URLs **must be convertible to [Strings](String)**.
    type Query: Query;

    /// Create a new [Downloader].
    fn new() -> Self;

    #[cfg(feature = "client")]
    /// Executes some [Query](Self::Query) to acquire its corresponding set of
    /// [Manifests](Manifest).
    async fn query(&self, query: Self::Query) -> Result<Vec<dash::Manifest>, Error> {
        let urls = query.to_urls().await.map_err(Error::custom)?;
        let mut manifests = Vec::new();
        for url_type in urls {
            let url = url_type.to_string();
            manifests.push(
                dash::Manifest::from_url(&url)
                    .await
                    .map_err(Error::custom)?,
            )
        }
        Ok(manifests)
    }
}

pub mod yt {
    //! Implementation of a maguro [Downloader] for YouTube.

    use async_trait::async_trait;
    use hyper::{self, body};
    use hyper_tls;
    use serde::Deserialize;
    use std::{
        error,
        fmt::{self, Display, Formatter},
        str::{self, FromStr},
        time::Duration,
    };

    /// Endpoint to request against.
    const ENDPOINT_URI: &'static str = "https://www.youtube.com/get_video_info";

    /// Query for a single YouTube video by its ID.
    #[derive(Debug, Clone)]
    pub struct YouTubeQuery(String);

    #[derive(Debug, Clone)]
    pub struct QueryError(String);

    impl error::Error for QueryError {}
    impl Display for QueryError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[async_trait]
    impl crate::Query for YouTubeQuery {
        type URL = String;
        type Error = crate::Error;

        async fn to_urls(&self) -> Result<Vec<Self::URL>, Self::Error> {
            let https = hyper_tls::HttpsConnector::new();
            let client = hyper::Client::builder().build::<_, hyper::Body>(https);

            let mut res = client
                .get(self.0.parse().unwrap())
                .await
                .map_err(crate::Error::custom)?;
            let resp: InfoWrapper = serde_urlencoded::from_bytes(
                body::to_bytes(res.body_mut())
                    .await
                    .map_err(crate::Error::custom)?
                    .to_vec()
                    .as_slice(),
            )
            .map_err(crate::Error::custom)?;
            let info: InfoResponse =
                serde_json::from_str(&resp.player_response).map_err(crate::Error::custom)?;

            Ok(vec![info.streaming_data.dash_manifest_url])
        }
    }

    impl FromStr for YouTubeQuery {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            todo!()
        }
    }

    pub struct Downloader;

    impl crate::Downloader for Downloader {
        type Query = YouTubeQuery;

        fn new() -> Self {
            Self {}
        }
    }

    #[derive(Deserialize, Clone, Debug)]
    /// Wrapper describing the outermost URL-encoded parameters of
    /// a get_video_info response.
    struct InfoWrapper {
        pub player_response: String,
    }

    #[derive(Deserialize, Clone, Debug)]
    /// YouTube get_video_info response.
    struct InfoResponse {
        #[serde(rename = "streamingData")]
        streaming_data: StreamingData,

        #[serde(rename = "videoDetails")]
        video_details: VideoDetails,
    }

    #[derive(Deserialize, Clone, Debug)]
    struct StreamingData {
        #[serde(rename = "dashManifestUrl")]
        dash_manifest_url: String,
    }

    #[derive(Deserialize, Clone, Debug)]
    /// Details about some YouTube video.
    pub struct VideoDetails {
        #[serde(rename = "videoId")]
        video_id: String,

        title: String,

        #[serde(rename = "author")]
        author: String,

        #[serde(
            rename = "lengthSeconds",
            deserialize_with = "crate::serde::duration::from_secs_option"
        )]
        approx_length: Option<Duration>,

        #[serde(rename = "viewCount")]
        views: u32,

        #[serde(rename = "isPrivate")]
        private: bool,

        #[serde(rename = "isLiveContent")]
        live: bool,
    }
}
