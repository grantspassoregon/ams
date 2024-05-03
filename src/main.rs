use aid::prelude::Clean;
use ams::prelude::{run, App};
// use polite::Polite;
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
    let icon = App::load_icon(include_bytes!("../data/gp_logo.png"))?;

    let event_loop = winit::event_loop::EventLoop::new()?;
    let window = winit::window::WindowBuilder::new()
        .with_title("AMS")
        .with_window_icon(Some(icon))
        .build(&event_loop)?;

    run(window, event_loop).await?;
    Ok(())
}
