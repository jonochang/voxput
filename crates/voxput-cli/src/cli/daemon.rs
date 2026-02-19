use voxput_core::errors::{Result, VoxputError};

// ---------------------------------------------------------------------------
// D-Bus proxy for voxputd
// ---------------------------------------------------------------------------

#[zbus::proxy(
    interface = "com.github.jonochang.Voxput1",
    default_service = "com.github.jonochang.Voxput",
    default_path = "/com/github/jonochang/Voxput"
)]
trait VoxputDaemon {
    async fn start_recording(&self) -> zbus::Result<()>;
    async fn stop_recording(&self) -> zbus::Result<()>;
    async fn toggle(&self) -> zbus::Result<()>;
    async fn get_status(&self) -> zbus::Result<(String, String, String)>;
}

async fn connect() -> Result<VoxputDaemonProxy<'static>> {
    let conn = zbus::Connection::session().await.map_err(|e| {
        VoxputError::Config(format!(
            "Could not connect to D-Bus session bus: {e}\n\
             Hint: make sure voxputd is running (`systemctl --user start voxputd`)"
        ))
    })?;
    VoxputDaemonProxy::new(&conn).await.map_err(|e| {
        VoxputError::Config(format!(
            "Could not connect to voxputd: {e}\n\
             Hint: start the daemon with `voxputd` or `systemctl --user start voxputd`"
        ))
    })
}

// ---------------------------------------------------------------------------
// Subcommand args
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct StartArgs {}

#[derive(Debug, clap::Args)]
pub struct StopArgs {}

#[derive(Debug, clap::Args)]
pub struct ToggleArgs {}

#[derive(Debug, clap::Args)]
pub struct StatusArgs {
    /// Print status as JSON.
    #[arg(long)]
    pub json: bool,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn run_start(_args: &StartArgs) -> Result<()> {
    let proxy = connect().await?;
    proxy.start_recording().await.map_err(|e| {
        VoxputError::Config(format!("start_recording failed: {e}"))
    })?;
    eprintln!("Recording started.");
    Ok(())
}

pub async fn run_stop(_args: &StopArgs) -> Result<()> {
    let proxy = connect().await?;
    proxy.stop_recording().await.map_err(|e| {
        VoxputError::Config(format!("stop_recording failed: {e}"))
    })?;
    eprintln!("Recording stopped.");
    Ok(())
}

pub async fn run_toggle(_args: &ToggleArgs) -> Result<()> {
    let proxy = connect().await?;
    proxy.toggle().await.map_err(|e| {
        VoxputError::Config(format!("toggle failed: {e}"))
    })?;
    Ok(())
}

pub async fn run_status(args: &StatusArgs) -> Result<()> {
    let proxy = connect().await?;
    let (state, transcript, error) = proxy.get_status().await.map_err(|e| {
        VoxputError::Config(format!("get_status failed: {e}"))
    })?;

    if args.json {
        let obj = serde_json::json!({
            "state": state,
            "transcript": transcript,
            "error": error,
        });
        println!("{}", serde_json::to_string_pretty(&obj)?);
    } else {
        println!("state:      {state}");
        if !transcript.is_empty() {
            println!("transcript: {transcript}");
        }
        if !error.is_empty() {
            println!("error:      {error}");
        }
    }
    Ok(())
}
