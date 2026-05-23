mod app;
mod weather;

use anyhow::Result;
use nestrs_core::App;
use nestrs_http::HttpTransport;
use nestrs_telemetry::Telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry = Telemetry::init("mcp")?;

    App::new::<app::AppModule>()
        .transport(HttpTransport::new().bind("0.0.0.0:3003"))
        .run()
        .await
}
