use eyre::{eyre, Context};
use lofty::{Tag, TagExt};
use std::path::Path;

use crate::{
    config::CONFIG,
    interface::metadata::{write_metadata, Metadata},
};

#[tracing::instrument(skip(old_tag))]
pub(super) async fn save_file(
    source_path: &Path,
    target_path: &Path,
    mut old_tag: Tag,
    metadata: Metadata,
) -> eyre::Result<()> {
    if let Ok(exist) = tokio::fs::try_exists(&target_path).await {
        if exist && !CONFIG.read().overwrite {
            return Err(eyre!("File already exists! Skipping..."));
        }
    }
    tokio::fs::create_dir_all(target_path.parent().unwrap()).await?;
    tokio::fs::copy(source_path, &target_path).await?;

    // let dummy_tag = Tag::new(old_tag.tag_type());
    // dbg!("Saving tag!!!!!!!!!");
    // dummy_tag.save_to_path(source_path).wrap_err("Dummy!")?;

    write_metadata(&mut old_tag, metadata);
    old_tag
        .save_to_path(target_path)
        .wrap_err("Failed to write tag")?;

    Ok(())
}
