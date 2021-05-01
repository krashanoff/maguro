//! # maguro
//!
//! An async library for downloading and streaming media, with
//! out-of-the-box support for YouTube.
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

use ::serde::{Deserialize, Serialize};
use hyper::{
    body::{self, HttpBody},
    Client,
};
use hyper_tls::HttpsConnector;
use std::{
    error,
    fmt::{self, Display},
    str,
    time::Duration,
};
use tokio::{fs::File, io::AsyncWriteExt};

pub mod serde;

/// Endpoint to request against.
const ENDPOINT_URI: &'static str = "https://www.youtube.com/get_video_info";

/// Form an endpoint URI for the given video ID.
fn endpoint_from_id<T: Display>(id: T) -> String {
    format!("{}?video_id={}", ENDPOINT_URI, id)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Describes a single streaming format for a YouTube video.
pub struct Format {
    itag: u32,
    url: String,

    // Width and height are optional in the case formats
    // are audio only.
    width: Option<u32>,
    height: Option<u32>,

    #[serde(rename = "mimeType")]
    mime_type: String,

    #[serde(
        default,
        rename = "contentLength",
        deserialize_with = "serde::u32::from_str_option"
    )]
    // A stream may not have a defined size.
    content_length: Option<u32>,

    quality: String,
    fps: Option<u32>,

    #[serde(
        default,
        rename = "approxDurationMs",
        deserialize_with = "serde::duration::from_millis_option"
    )]
    // A stream may not have a defined length.
    approx_duration: Option<Duration>,
}

impl Format {
    /// Whether the given streaming format is a video.
    pub fn is_video(&self) -> bool {
        match self.width {
            Some(_) => true,
            None => false,
        }
    }

    pub fn itag(&self) -> u32 {
        self.itag
    }

    pub fn size(&self) -> Option<u32> {
        self.content_length.clone()
    }

    /// Read the entire YouTube video into a vector.
    pub async fn to_vec(&self) -> Result<Vec<u8>, Box<dyn error::Error + Send + Sync>> {
        self.to_vec_callback(|_| Ok(())).await
    }

    /// Downloads the entire YouTube video in chunks with the given closure.
    /// On receipt of a new chunk of bytes, it calls the closure.
    pub async fn to_vec_callback<T>(
        &self,
        on_chunk: T,
    ) -> Result<Vec<u8>, Box<dyn error::Error + Send + Sync>>
    where
        T: Fn(Vec<u8>) -> Result<(), Box<dyn error::Error + Send + Sync>>,
    {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        let mut res = client.get(self.url.parse().unwrap()).await.unwrap();

        let mut v: Vec<u8> = Vec::new();
        while let Some(chunk) = res.body_mut().data().await {
            let as_bytes: Vec<u8> = chunk?.iter().cloned().collect();
            on_chunk(as_bytes.clone())?;
            v.extend(as_bytes.iter());
        }
        Ok(v)
    }

    /// Downloads the entire YouTube video into a `File`.
    pub async fn download(
        &self,
        dest: &mut File,
    ) -> Result<(), Box<dyn error::Error + Send + Sync>> {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        let mut res = client.get(self.url.parse().unwrap()).await.unwrap();

        while let Some(chunk) = res.body_mut().data().await {
            dest.write(&chunk?).await?;
        }

        Ok(())
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "itag: {:03}\tQuality: {}\tMime Type: {}",
            self.itag, self.quality, self.mime_type
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// The set of sources available to download a YouTube
/// video with.
pub struct StreamingData {
    #[serde(
        rename = "expiresInSeconds",
        deserialize_with = "serde::duration::from_secs"
    )]
    expires_in_seconds: Duration,

    // In the case of streams, the `formats` field is empty.
    formats: Option<Vec<Format>>,

    #[serde(rename = "adaptiveFormats")]
    adaptive_formats: Vec<Format>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Details about some YouTube video.
pub struct VideoDetails {
    #[serde(rename = "videoId")]
    video_id: String,

    title: String,

    #[serde(rename = "author")]
    author: String,

    #[serde(
        rename = "lengthSeconds",
        deserialize_with = "serde::duration::from_secs_option"
    )]
    approx_length: Option<Duration>,

    #[serde(rename = "viewCount", deserialize_with = "serde::u32::from_str")]
    views: u32,

    #[serde(rename = "isPrivate")]
    private: bool,

    #[serde(rename = "isLiveContent")]
    live: bool,
}

impl VideoDetails {
    pub fn id(&self) -> String {
        self.video_id.clone()
    }
}

#[derive(Deserialize, Clone, Debug)]
/// YouTube get_video_info response.
pub struct InfoResponse {
    #[serde(rename = "streamingData")]
    streaming_data: StreamingData,

    #[serde(rename = "videoDetails")]
    video_details: VideoDetails,
}

impl InfoResponse {
    pub fn formats(&self) -> Option<Vec<Format>> {
        self.streaming_data.formats.clone()
    }

    pub fn adaptive_formats(&self) -> Vec<Format> {
        self.streaming_data.adaptive_formats.clone()
    }

    pub fn details(&self) -> VideoDetails {
        self.video_details.clone()
    }

    /// Returns a vector of all formats available for the given
    /// video.
    pub fn all_formats(&self) -> Vec<Format> {
        if let Some(fmts) = self.formats() {
            return fmts
                .iter()
                .cloned()
                .chain(self.adaptive_formats().iter().cloned())
                .collect();
        }
        self.adaptive_formats().iter().cloned().collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Wrapper describing the outermost URL-encoded parameters of
/// a get_video_info response.
struct InfoWrapper {
    pub player_response: String,
}

/// Acquires the [InfoResponse] struct for a given video ID.
pub async fn get_video_info(id: &str) -> Result<InfoResponse, Box<dyn error::Error>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let mut res = client
        .get(endpoint_from_id(id).parse().unwrap())
        .await
        .unwrap();
    let body = body::to_bytes(res.body_mut()).await.unwrap();

    let stream_info: InfoResponse =
        serde_json::from_str(&serde_urlencoded::from_bytes::<InfoWrapper>(&body)?.player_response)?;
    Ok(stream_info)
}
