//! An ergonomic Rust translation of DASH-MPD, as specified by
//! [standards.iso.org](https://standards.iso.org/ittf/PubliclyAvailableStandards/MPEG-DASH_schema_files/DASH-MPD-edition2.xsd).
//!
//! At present, only the portions that are necessary for maguro to
//! function are translated. In the future, this process should ideally be
//! automated.

use hyper::{self, body};
use hyper_tls;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, error, iter::FromIterator, str};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "MPD")]
/// Entry point; root of a DASH-MPEG manifest.
pub struct Manifest {
    #[serde(rename = "Period")]
    periods: Vec<Period>,

    #[serde(rename = "type")]
    mpd_type: String,
}

impl Manifest {
    #[cfg(feature = "client")]
    /// Acquires a [Manifest] from the provided URL source.
    pub async fn from_url<T: ToString>(
        url: &T,
    ) -> Result<Self, Box<dyn error::Error + Send + Sync>> {
        let https = hyper_tls::HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);

        let mut res = client.get(url.to_string().parse().unwrap()).await?;
        let body = String::from_iter(
            body::to_bytes(res.body_mut())
                .await?
                .to_vec()
                .iter()
                .map(|e| char::from(e.clone()))
                .filter(|e| e.is_ascii()),
        );

        Ok(Self::try_from(body.as_str())?)
    }
}

impl TryFrom<&str> for Manifest {
    type Error = serde_xml_rs::Error;

    /// Attempt to parse an XML [&str] into a [Manifest].
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        serde_xml_rs::from_str(s)
    }
}

impl Manifest {
    /// Available [AdaptationSets](AdaptationSet) for the given media's manifest.
    pub fn streams(&self) -> Vec<AdaptationSet> {
        self.period.adaptation_sets.clone()
    }

    #[cfg(feature = "client")]
    /// Acquires a [Manifest] from the provided URL source.
    pub async fn from_url<T: ToString>(
        url: &T,
    ) -> Result<Self, Box<dyn error::Error + Send + Sync>> {
        let https = hyper_tls::HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);

        let mut res = client.get(url.to_string().parse().unwrap()).await?;
        let body = body::to_bytes(res.body_mut()).await?.to_vec();

        // TODO: remove `unwrap`.
        Ok(Self::try_from(str::from_utf8(body.as_slice())?).unwrap())
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
/// Set of formats available to stream for the given [MIME](mime::Mime) type.
pub struct AdaptationSet {
    #[serde(rename = "segmentAlignment")]
    segment_alignment: Option<bool>,

    id: Option<u32>,

    #[serde(
        default,
        rename = "mimeType",
        deserialize_with = "crate::serde::mime::option_from_str",
        serialize_with = "crate::serde::mime::option_to_str",
    )]
    mime_type: Option<mime::Mime>,

    #[serde(rename = "subsegmentAlignment")]
    subsegment_alignment: Option<bool>,

    #[serde(rename = "Role")]
    role: Option<Role>,

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
    // RepresentationBaseType
    profiles: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    sar: Option<String>,

    #[serde(rename = "frameRate")]
    frame_rate: Option<u32>,

    #[serde(
        default,
        rename = "mimeType",
        deserialize_with = "crate::serde::mime::option_from_str",
        serialize_with = "crate::serde::mime::option_to_str",
    )]
    mime_type: Option<mime::Mime>,

    // Subelements
    #[serde(rename = "BaseURL")]
    base_urls: Option<Vec<String>>,

    #[serde(rename = "SubRepresentation")]
    sub_representations: Option<Vec<Representation>>,

    // TODO
    #[serde(rename = "SegmentBase")]
    segment_base: Option<SegmentURL>,

    #[serde(rename = "SegmentList")]
    segment_list: Option<SegmentList>,

    #[serde(rename = "SegmentTemplate")]
    segment_template: Option<SegmentURL>,

    // Attributes
    id: String,
    bandwidth: u32,

    #[serde(rename = "qualityRanking")]
    quality_ranking: Option<u32>,

    #[serde(rename = "dependencyId")]
    dependency_id: Option<String>,

    #[serde(rename = "mediaStreamStructureId")]
    media_stream_structure_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    /// Tests against a known simple multi-resolution manifest.
    async fn from_url() {
        match Manifest::from_url(
            &"https://dash.akamaized.net/dash264/TestCases/2c/qualcomm/1/MultiResMPEG2.mpd",
        )
        .await
        {
            Ok(m) => m,
            Err(e) => {
                println!("Failed to fetch valid manifest! {}", e);
                assert!(false);
                return;
            }
        };
    }
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
