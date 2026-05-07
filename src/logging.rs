use once_cell::sync::OnceCell;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
static LOG_SET: OnceCell<()> = OnceCell::new();
pub fn init_logging() {
    LOG_SET.get_or_init(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(filter)
            .init();
    });
}
