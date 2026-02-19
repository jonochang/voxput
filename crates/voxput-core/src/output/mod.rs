pub mod clipboard;
pub mod stdout;

use crate::errors::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputTarget {
    #[default]
    Stdout,
    Clipboard,
    Both,
}

pub trait OutputSink: Send + Sync {
    fn write(&self, text: &str) -> Result<()>;
}

pub fn create_sink(target: OutputTarget) -> Box<dyn OutputSink> {
    match target {
        OutputTarget::Stdout => Box::new(stdout::StdoutSink),
        OutputTarget::Clipboard => Box::new(clipboard::ClipboardSink),
        OutputTarget::Both => Box::new(BothSink),
    }
}

struct BothSink;

impl OutputSink for BothSink {
    fn write(&self, text: &str) -> Result<()> {
        stdout::StdoutSink.write(text)?;
        clipboard::ClipboardSink.write(text)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdout_sink_does_not_error() {
        stdout::StdoutSink.write("test").expect("stdout sink should not error");
    }
}
