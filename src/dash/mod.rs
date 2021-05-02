//! An ergonomic Rust translation of the DASH-MPD.xsd found at
//! [standards.iso.org](https://standards.iso.org/ittf/PubliclyAvailableStandards/MPEG-DASH_schema_files/DASH-MPD.xsd).
//!
//! At present, only the portions that are necessary for maguro to
//! function are translated. In the future, this process should ideally be
//! automated.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MPD {
    #[serde(rename = "Period")]
    period: Period,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Period {
    #[serde(default, rename = "AdaptationSet")]
    adaptation_sets: Vec<AdaptationSet>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AdaptationSet {
    id: u32,

    #[serde(rename = "mimeType")]
    // TODO: make into a mime::Mime type.
    mime_type: String,

    #[serde(rename = "subsegmentAlignment")]
    subsegment_alignment: bool,

    #[serde(rename = "Role")]
    role: Role,

    #[serde(rename = "Representation")]
    representations: Vec<Representation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Role {
    #[serde(rename = "schemeIdUri")]
    scheme_id_uri: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SegmentURL {
    media: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Initialization {
    #[serde(rename = "sourceURL")]
    source_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The list of segments
struct SegmentList {
    #[serde(rename = "Initialization")]
    initialization: Initialization,

    #[serde(rename = "SegmentURL")]
    segment_urls: Vec<SegmentURL>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Representation {
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
