use anyhow::Result;
use reqwest::{header::HeaderMap, Response};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::Deserialize;

use crate::domain::errors::LastfmError;
use crate::domain::models::Tag as DomainTag;

static USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:100.0) Gecko/20100101 Firefox/100.0";

pub async fn fetch_track_tags(api_key: &str, track: &str, artist: &str) -> Result<Vec<DomainTag>> {
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", USER_AGENT.parse().unwrap());

    let url = format!(
        "https://ws.audioscrobbler.com/2.0/?method=track.gettoptags&artist={}&track={}&api_key={}&format=json",
        artist, track, api_key
    );

    let res = http_get(url.as_str(), &headers).await?;
    match res.json::<TrackTagsResponse>().await {
        Ok(res) => Ok(extract_tags(res)),
        Err(_) => Err(anyhow::Error::new(LastfmError::TrackTagsJsonParse(
            String::from("TrackTagsResponse"),
        ))),
    }
}

async fn http_get(url: &str, headers: &HeaderMap) -> Result<Response> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let res = client
        .get(url)
        .headers(headers.clone())
        .send()
        .await
        .map_err(LastfmError::TrackTagsResponse)?;

    if !res.status().is_success() {
        let err = anyhow::Error::new(LastfmError::Api);
        tracing::error!("{} requesting `{}`", err, url);
        return Err(err);
    }

    Ok(res)
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrackTagsResponse {
    #[serde(rename = "toptags")]
    pub track_tags: TrackTags,
}

fn extract_tags(tt: TrackTagsResponse) -> Vec<DomainTag> {
    let mut tags: Vec<Tag> = tt.track_tags.tags;
    tags.sort_by(|a, b| a.count.cmp(&b.count));
    tags.truncate(3);
    tags.into_iter()
        .map(|tag| DomainTag { id: tag.name })
        .collect()
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrackTags {
    #[serde(rename = "tag")]
    pub tags: Vec<Tag>,
    #[serde(rename = "@attr")]
    pub attr: Attr,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Tag {
    pub count: i64,
    pub name: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Attr {
    pub artist: String,
    pub track: String,
}
