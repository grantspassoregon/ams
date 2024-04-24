use ams::prelude::run;
use polite::Polite;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


#[tokio::main]
async fn main() -> Polite<()> {
    if tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ams=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init().is_ok() {}; 
    tracing::info!("Subscriber initialized.");

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = winit::window::WindowBuilder::new()
        .with_title("AMS")
        .build(&event_loop)?;

    run(window, event_loop).await;
    Ok(())
}
