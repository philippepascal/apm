use std::path::PathBuf;
use crate::AppError;

pub async fn blocking<F, T>(f: F) -> Result<T, AppError>
where
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(AppError::from)?
        .map_err(AppError::from)
}

pub async fn load_config(root: PathBuf) -> Result<apm_core::config::Config, AppError> {
    blocking(move || apm_core::config::Config::load(&root)).await
}
