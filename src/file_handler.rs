use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

pub struct FileHandler {
    upload_dir: PathBuf,
}

impl FileHandler {
    pub fn new(upload_dir: PathBuf) -> Self {
        Self { upload_dir }
    }

    pub async fn save_file_chunk(
        &self,
        filename: &str,
        data: &[u8],
        chunk_id: u32,
        total_chunks: u32,
    ) -> Result<bool> {
        let file_id = Uuid::new_v4();
        let chunk_path = self.upload_dir.join(format!("{}_{}_{}", file_id, chunk_id, filename));
        
        fs::write(&chunk_path, data).await?;

        if chunk_id + 1 == total_chunks {
            self.merge_chunks(file_id, filename, total_chunks).await?;
            return Ok(true);
        }

        Ok(false)
    }

    async fn merge_chunks(&self, file_id: Uuid, filename: &str, total_chunks: u32) -> Result<()> {
        let final_path = self.upload_dir.join(filename);
        let mut final_file = fs::File::create(&final_path).await?;

        for i in 0..total_chunks {
            let chunk_path = self.upload_dir
                .join(format!("{}_{}_{}", file_id, i, filename));
            let chunk_data = fs::read(&chunk_path).await?;
            final_file.write_all(&chunk_data).await?;
            fs::remove_file(chunk_path).await?;
        }

        Ok(())
    }
}
