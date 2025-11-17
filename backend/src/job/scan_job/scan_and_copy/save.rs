use eyre::{eyre, Context};
use lofty::{
    config::WriteOptions,
    tag::{Tag, TagExt},
};
use std::path::Path;

use crate::config::CONFIG;

#[tracing::instrument(skip(new_tag))]
pub(super) async fn save_file(
    source_path: &Path,
    target_path: &Path,
    new_tag: Tag,
) -> eyre::Result<()> {
    if let Ok(exist) = tokio::fs::try_exists(&target_path).await {
        if exist && !CONFIG.read().overwrite {
            return Err(eyre!("File already exists! Skipping..."));
        }
    }
    tokio::fs::create_dir_all(target_path.parent().unwrap()).await?;
    tokio::fs::copy(source_path, &target_path).await?;

    new_tag
        .save_to_path(target_path, WriteOptions::new())
        .wrap_err("Failed to write tag")?;

    Ok(())
}
