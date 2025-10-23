// Terminal User Interface module

pub mod app;
pub mod events;
pub mod ui;

pub use app::{App, AppConfig};
pub use events::handle_events;
pub use ui::render;
