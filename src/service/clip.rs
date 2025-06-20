use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use apalis::prelude::{Data, MemoryStorage, MessageQueue};
use sea_orm::IntoActiveModel;
use sea_orm::Set;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::{fs::File, io::BufWriter};
use tracing::{debug, error, trace};
use uuid::Uuid;

use crate::core::entity::clip::{self, Entity as Clip};
use crate::core::entity::user;
use crate::core::storage::Storage;

#[derive(Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub is_streaming: bool,
    pub total_clips: u64,
    pub unreviewed: u64,
}

#[derive(Clone)]
pub struct ClipService {
    tmp_dir: PathBuf,
    db: DatabaseConnection,
    storage: Arc<Storage>,
    queue: Arc<Mutex<MemoryStorage<ProcessJob>>>,
}

impl ClipService {
    pub fn new(
        tmp_dir: PathBuf,
        db: DatabaseConnection,
        storage: Arc<Storage>,
        queue: MemoryStorage<ProcessJob>,
    ) -> Self {
        Self {
            tmp_dir,
            db,
            storage,
            queue: Arc::new(Mutex::new(queue)),
        }
    }

    pub async fn save_clip_to_tmp<'a, R>(
        &'a self,
        reader: &'a mut R,
        ext: &'a str,
    ) -> anyhow::Result<(Uuid, PathBuf)>
    where
        R: AsyncRead + Unpin + ?Sized,
    {
        let uuid = uuid::Uuid::new_v4();
        let file_path = self.tmp_dir.join(format!("{uuid}.{ext}"));
        let mut file = BufWriter::new(File::create(&file_path).await?);
        tokio::io::copy(reader, &mut file).await?;
        Ok((uuid, file_path))
    }

    pub async fn create_clip(
        &self,
        user_id: i64,
        req: clip::Model,
        file: PathBuf,
    ) -> anyhow::Result<clip::Model> {
        let clip = clip::ActiveModel {
            uuid: Set(req.uuid),
            title: Set(req.title.clone()),
            vup: Set(req.vup.clone()),
            song: Set(req.song.clone()),
            user_id: Set(user_id),
            ..Default::default()
        }
        .insert(&self.db)
        .await?;

        let mut queue = self.queue.lock().await;
        match queue
            .enqueue(ProcessJob {
                clip: clip.clone(),
                input_path: file,
            })
            .await
        {
            Ok(_) => debug!("Clip job enqueued successfully"),
            Err(_) => error!("Failed to enqueue clip job"),
        }

        Ok(clip)
    }

    pub async fn process_clip(&self, clip: clip::Model, file: PathBuf) -> anyhow::Result<()> {
        trace!("Processing clip: {}, path: {}", clip.uuid, file.display());
        let mut active_clip = clip.into_active_model();
        active_clip.status = Set(clip::Status::Processing);
        let clip = active_clip.update(&self.db).await?;

        let output_path = self.tmp_dir.join(format!("{}_processed.mp4", clip.uuid));

        match ClipService::transcode_and_normalize(&file, &output_path).await {
            Ok(_) => {
                debug!("Clip {} processed successfully", clip.uuid);
            }
            Err(e) => {
                error!("Failed to process clip {}: {}", clip.uuid, e);
                let mut active_clip = clip.clone().into_active_model();
                active_clip.status = Set(clip::Status::Failed);
                active_clip
                    .update(&self.db)
                    .await
                    .map_err(|e| {
                        error!("Failed to update clip status to failed: {}", e);
                    })
                    .ok();
                return Err(e.into());
            }
        }

        match self
            .storage
            .store_file(format!("{}.mp4", clip.uuid.to_string()), &output_path)
            .await
        {
            Ok(_) => {
                debug!("Clip {} stored successfully", clip.uuid);
            }
            Err(e) => {
                error!("Failed to store clip {}: {}", clip.uuid, e);
                let mut active_clip = clip.clone().into_active_model();
                active_clip.status = Set(clip::Status::Failed);
                active_clip
                    .update(&self.db)
                    .await
                    .map_err(|e| {
                        error!("Failed to update clip status to failed: {}", e);
                    })
                    .ok();
                return Err(e.into());
            }
        }

        // remove the temporary file after storing
        tokio::fs::remove_file(file)
            .await
            .map_err(|e| {
                error!("Failed to remove clip file: {}", e);
            })
            .ok();
        tokio::fs::remove_file(output_path)
            .await
            .map_err(|e| {
                error!("Failed to remove processed clip file: {}", e);
            })
            .ok();

        let mut active_clip = clip.into_active_model();
        active_clip.status = Set(clip::Status::Reviewing);
        active_clip
            .update(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to update clip status: {}", e);
            })
            .ok();

        Ok(())
    }

    pub async fn list_clips_by_user(&self, user_id: i64) -> anyhow::Result<Vec<clip::Model>> {
        trace!("Listing clips for user {}", user_id);
        let clips = Clip::find()
            .filter(clip::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch clips for user {}: {}", user_id, e);
                e
            })?;
        debug!("Fetched {} clips for user {}", clips.len(), user_id);
        Ok(clips)
    }

    pub async fn get_clip_by_uuid(
        &self,
        user_id: i64,
        uuid: Uuid,
    ) -> anyhow::Result<Option<clip::Model>> {
        trace!("Fetching clip by UUID: {}", uuid.to_string());
        let clip = Clip::find()
            .filter(clip::Column::Uuid.eq(uuid))
            .filter(clip::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?;

        Ok(clip)
    }

    async fn _get_clip_by_id(&self, id: i64) -> anyhow::Result<clip::Model> {
        trace!("Fetching clip by ID: {}", id);
        let clip = Clip::find()
            .filter(clip::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch clip by ID {}: {}", id, e);
                e
            })?
            .ok_or_else(|| {
                error!("Clip not found for user ID: {}", id);
                anyhow::anyhow!("Clip not found")
            })?;
        debug!("Fetched clip: {}", clip.uuid.to_string());
        Ok(clip)
    }

    pub async fn update_clip(
        &self,
        user: user::Model,
        req: clip::Model,
    ) -> anyhow::Result<Option<clip::Model>> {
        trace!("Updating clip {} for {}", req.uuid.to_string(), user.id);
        let clip = self
            .get_clip_by_uuid(user.id, req.uuid)
            .await
            .map_err(|e| {
                error!("Failed to verify clip ownership: {}", e);
                anyhow!("Failed to verify clip ownership: {}", e)
            })?;

        let clip = match clip {
            Some(c) => c,
            None => {
                return Ok(None);
            }
        };

        let mut active_clip = clip.clone().into_active_model();
        active_clip.title = Set(req.title);
        active_clip.vup = Set(req.vup);
        active_clip.song = Set(req.song);
        let clip = active_clip
            .update(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to update clip: {}", e))?;
        debug!("Clip updated successfully: {}", clip.uuid.to_string());
        Ok(Some(clip))
    }

    pub async fn set_clip_reviewed(
        &self,
        user: user::Model,
        req: clip::Model,
    ) -> anyhow::Result<Option<clip::Model>> {
        trace!(
            "Setting clip {} for {} as reviewed",
            req.uuid.to_string(),
            user.id
        );

        let clip = self
            .get_clip_by_uuid(user.id, req.uuid)
            .await
            .map_err(|e| {
                error!("Failed to verify clip ownership: {}", e);
                anyhow!("Failed to verify clip ownership: {}", e)
            })?;

        let clip = match clip {
            Some(c) => c,
            None => {
                error!("Clip {} not found for user ID: {}", req.uuid, user.id);
                return Ok(None);
            }
        };
        if clip.status != clip::Status::Reviewing {
            return Err(anyhow!("Clip is not in reviewing status"));
        }

        let mut active_clip = clip.into_active_model();
        active_clip.status = Set(clip::Status::Reviewed);
        let clip = active_clip.update(&self.db).await.map_err(|e| {
            error!("Failed to set clip as reviewed: {}", e);
            e
        })?;
        debug!(
            "Clip set as reviewed successfully: {}",
            clip.uuid.to_string()
        );
        Ok(Some(clip))
    }
    async fn transcode_and_normalize(
        input_path: &PathBuf,
        output_path: &PathBuf,
    ) -> anyhow::Result<()> {
        let status = Command::new("ffmpeg")
            .arg("-i")
            .arg(input_path)
            .arg("-af")
            .arg("loudnorm")
            .arg("-c:v")
            .arg("copy")
            // .arg("libx264")
            // .arg("-preset")
            // .arg("medium")
            .arg("-c:a")
            .arg("aac")
            .arg("-b:a")
            .arg("320k")
            .arg("-movflags")
            .arg("faststart")
            .arg(output_path)
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("FFmpeg command failed with status: {}", status);
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ProcessJob {
    pub clip: clip::Model,
    pub input_path: PathBuf,
}
pub async fn process_clip(job: ProcessJob, data: Data<Arc<ClipService>>) -> anyhow::Result<()> {
    debug!("Processing clip: {:?}", job.clip.uuid);
    data.process_clip(job.clip, job.input_path).await?;
    Ok(())
}
