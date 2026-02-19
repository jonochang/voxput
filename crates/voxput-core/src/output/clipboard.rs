use arboard::Clipboard;

use crate::errors::{Result, VoxputError};
use crate::output::OutputSink;

pub struct ClipboardSink;

impl OutputSink for ClipboardSink {
    fn write(&self, text: &str) -> Result<()> {
        let mut clipboard = Clipboard::new()
            .map_err(|e| VoxputError::Output(format!("Failed to access clipboard: {e}")))?;
        clipboard
            .set_text(text.to_string())
            .map_err(|e| VoxputError::Output(format!("Failed to write to clipboard: {e}")))?;
        eprintln!("Copied to clipboard.");
        Ok(())
    }
}
