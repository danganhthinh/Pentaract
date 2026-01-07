use reqwest::multipart;
use uuid::Uuid;

use crate::{
    common::types::ChatId,
    errors::{PentaractError, PentaractResult},
    services::storage_workers_scheduler::StorageWorkersScheduler,
};

use super::schemas::{DownloadBodySchema, UploadBodySchema, UploadSchema};

pub struct TelegramBotApi<'t> {
    base_url: &'t str,
    scheduler: StorageWorkersScheduler<'t>,
}

impl<'t> TelegramBotApi<'t> {
    pub fn new(base_url: &'t str, scheduler: StorageWorkersScheduler<'t>) -> Self {
        Self {
            base_url,
            scheduler,
        }
    }

    pub async fn upload(
        &self,
        file: &[u8],
        chat_id: ChatId,
        storage_id: Uuid,
    ) -> PentaractResult<UploadSchema> {
        let chat_id = {
            // Check if chat_id already has the -100 prefix (supergroup/channel format)
            if chat_id.to_string().starts_with("-100") {
                // Already in correct format, use as-is
                tracing::debug!("[Telegram API] Chat ID already in supergroup format: {}", chat_id);
                chat_id
            } else {
                // Apply transformation for regular group chats
                // inserting 100 between minus sign and chat id
                // https://stackoverflow.com/a/65965402/12255756
                let n = chat_id.abs().checked_ilog10().unwrap_or(0) + 1;
                let transformed = chat_id - (100 * ChatId::from(10).pow(n));
                tracing::debug!("[Telegram API] Transformed chat ID from {} to {}", chat_id, transformed);
                transformed
            }
        };

        let token = self.scheduler.get_token(storage_id).await?;
        let url = self.build_url("", "sendDocument", token);

        let file_part = multipart::Part::bytes(file.to_vec()).file_name("pentaract_chunk.bin");
        let form = multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .part("document", file_part);

        let response = reqwest::Client::new()
            .post(url)
            .multipart(form)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json::<UploadBodySchema>().await?.result.document)
        } else {
            let error_body = response.text().await.unwrap_or_else(|_| "Unable to read response body".to_string());
            tracing::error!("[Telegram API] {} - Response: {}", status, error_body);
            Err(PentaractError::TelegramAPIError(format!("{} - {}", status, error_body)))
        }
    }

    pub async fn download(
        &self,
        telegram_file_id: &str,
        storage_id: Uuid,
    ) -> PentaractResult<Vec<u8>> {
        // getting file path
        let token = self.scheduler.get_token(storage_id).await?;
        let url = self.build_url("", "getFile", token);
        // TODO: add retries with their number taking from env
        let body: DownloadBodySchema = reqwest::Client::new()
            .get(url)
            .query(&[("file_id", telegram_file_id)])
            .send()
            .await?
            .json()
            .await?;

        // downloading the file itself
        let token = self.scheduler.get_token(storage_id).await?;
        let url = self.build_url("file/", &body.result.file_path, token);
        let file = reqwest::get(url)
            .await?
            .bytes()
            .await
            .map(|file| file.to_vec())?;

        Ok(file)
    }

    /// Taking token by a value to force dropping it so it can be used only once
    #[inline]
    fn build_url(&self, pre: &str, relative: &str, token: String) -> String {
        format!("{}/{pre}bot{token}/{relative}", self.base_url)
    }
}
