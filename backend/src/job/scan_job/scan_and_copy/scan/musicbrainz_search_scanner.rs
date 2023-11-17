use eyre::eyre;
use lofty::{Accessor, Tag};

use crate::{api::musicbrainz::MusicbrainzClient, job::scan_job::scan_and_copy::ScannerInfo};

use super::ScannerRes;

#[tracing::instrument(skip(crr_tag))]
pub(super) async fn musicbrainz_search_scanner(crr_tag: &Tag) -> Result<ScannerRes, eyre::Report> {
    let mb_client = MusicbrainzClient::new();

    let title = crr_tag.title().ok_or_else(|| eyre!("No title tag"))?;

    let query = format!(r#"recording:"{}""#, title);

    let res = mb_client.recording_search(&query).await?;

    Ok(ScannerRes {
        log: ScannerInfo::MusicbrainzSearch,
        recordings: res.recordings,
    })
}
