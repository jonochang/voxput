use crate::errors::Result;
use crate::output::OutputSink;

pub struct StdoutSink;

impl OutputSink for StdoutSink {
    fn write(&self, text: &str) -> Result<()> {
        println!("{text}");
        Ok(())
    }
}
