use std::path::Path;

use super::types::LayoutData;
use crate::error::AppError;

pub fn read_layout(path: &Path) -> Result<LayoutData, AppError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::LayoutError(format!("Failed to read {}: {}", path.display(), e)))?;
    let data: LayoutData = toml::from_str(&content)
        .map_err(|e| AppError::LayoutError(format!("Failed to parse {}: {}", path.display(), e)))?;

    Ok(data)
}

pub fn write_layout(path: &Path, data: &LayoutData) -> Result<(), AppError> {
    let content = toml::to_string_pretty(data)
        .map_err(|e| AppError::LayoutError(format!("Failed to serialize layout data: {}", e)))?;
    std::fs::write(path, content)
        .map_err(|e| AppError::LayoutError(format!("Failed to write {}: {}", path.display(), e)))?;

    Ok(())
}
