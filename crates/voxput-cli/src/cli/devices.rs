use clap::Args;
use voxput_core::audio::cpal_backend::CpalBackend;
use voxput_core::audio::AudioBackend;
use voxput_core::errors::Result;

#[derive(Debug, Args)]
pub struct DevicesArgs {
    /// Print output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &DevicesArgs) -> Result<()> {
    let backend = CpalBackend;
    let devices = backend.list_devices()?;

    if args.json {
        let json = serde_json::to_string_pretty(&devices)
            .map_err(voxput_core::errors::VoxputError::Json)?;
        println!("{json}");
    } else {
        for device in &devices {
            let marker = if device.is_default { " (default)" } else { "" };
            println!("{}{}", device.name, marker);
        }
    }

    Ok(())
}
