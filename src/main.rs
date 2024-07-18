use aid::prelude::Clean;
use ams::app;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Clean<()> {
    if tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ams=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .is_ok()
    {};
    tracing::info!("Subscriber initialized.");

    let (app, event_loop) = app::App::boot().await?;
    app.run(event_loop).await?;
    Ok(())
}
