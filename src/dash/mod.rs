//! An ergonomic Rust translation of DASH-MPD, as specified by
//! [standards.iso.org](https://standards.iso.org/ittf/PubliclyAvailableStandards/MPEG-DASH_schema_files/DASH-MPD.xsd).
//!
//! At present, only the portions that are necessary for maguro to
//! function are translated. In the future, this process should ideally be
//! automated.

use crate::serde::mime as mime_ext;
use hyper::{self, body::HttpBody};
use hyper_tls;
use mime;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, convert::TryFrom, error, fmt};
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "MPD")]
/// Root of a DASH-MPEG manifest.
pub struct Manifest {
    #[serde(rename = "Period")]
    period: Period,
}

impl Manifest {
    /// Available [AdaptationSets](AdaptationSet) for the given media's manifest.
    pub fn streams(&self) -> Vec<AdaptationSet> {
        self.period.adaptation_sets.clone()
    }
}

impl TryFrom<&str> for Manifest {
    type Error = serde_xml_rs::Error;

    /// Attempt to parse an XML [&str] into a [Manifest].
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        serde_xml_rs::from_str(s)
    }
}

impl IntoIterator for Manifest {
    type Item = AdaptationSet;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.streams().into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Period {
    #[serde(default, rename = "AdaptationSet")]
    pub adaptation_sets: Vec<AdaptationSet>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Set of formats available to stream to the given [MIME](mime::Mime) type.
pub struct AdaptationSet {
    id: u32,

    #[serde(
        rename = "mimeType",
        deserialize_with = "mime_ext::to_mime",
        serialize_with = "mime_ext::to_str"
    )]
    mime_type: mime::Mime,

    #[serde(rename = "subsegmentAlignment")]
    subsegment_alignment: bool,

    #[serde(rename = "Role")]
    role: Role,

    #[serde(rename = "Representation")]
    representations: Vec<Representation>,
}

impl fmt::Display for AdaptationSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Adaptation Set: id {}; {}", self.id, self.mime_type)
    }
}

impl PartialOrd for AdaptationSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AdaptationSet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialEq for AdaptationSet {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AdaptationSet {}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Role {
    #[serde(rename = "schemeIdUri")]
    pub scheme_id_uri: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
struct SegmentURL {
    pub media: String,
}

impl PartialOrd for SegmentURL {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SegmentURL {
    fn cmp(&self, other: &Self) -> Ordering {
        self.media.cmp(&other.media)
    }
}

impl PartialEq for SegmentURL {
    fn eq(&self, other: &Self) -> bool {
        self.media == other.media
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Initialization {
    #[serde(rename = "sourceURL")]
    pub source_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The list of segments
struct SegmentList {
    #[serde(rename = "Initialization")]
    pub initialization: Initialization,

    #[serde(rename = "SegmentURL")]
    pub segment_urls: Vec<SegmentURL>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A streaming format for some adaptation.
pub struct Representation {
    id: u32,

    codecs: String,

    #[serde(rename = "audioSamplingRate")]
    audio_sampling_rate: Option<u32>,

    width: Option<u32>,
    height: Option<u32>,

    #[serde(rename = "frameRate")]
    frame_rate: Option<u32>,

    #[serde(rename = "BaseURL")]
    base_url: String,

    #[serde(rename = "SegmentList")]
    segment_list: SegmentList,
}

impl Representation {
    /// Vector of the URLs of each chunk **in the order they should
    /// be downloaded in**.
    pub fn segment_urls(&self) -> Vec<String> {
        let mut urls = Vec::new();

        urls.push(format!(
            "{}/{}",
            self.base_url, self.segment_list.initialization.source_url
        ));
        for segment in self.segment_list.segment_urls.iter() {
            urls.push(format!("{}/{}", self.base_url, segment.media));
        }

        urls
    }

    /// Asynchronously downloads the given [Representation] to an [AsyncWriter](AsyncWrite).
    pub async fn download<T: AsyncWrite + Unpin>(
        &self,
        writer: &mut T,
    ) -> Result<(), Box<dyn error::Error + Send + Sync>> {
        let https = hyper_tls::HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);

        for segment_url in self.segment_urls() {
            let mut res = client.get(segment_url.parse().unwrap()).await?;

            while let Some(chunk) = res.body_mut().data().await {
                writer.write(&chunk?).await?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Representation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {} - ({:?}x{:?})",
            self.id, self.codecs, self.width, self.height
        )
    }
}

impl PartialOrd for Representation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Representation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialEq for Representation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Representation {}
