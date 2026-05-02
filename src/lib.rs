#![warn(clippy::all, rust_2018_idioms)]

mod app;

#[cfg(target_arch = "wasm32")]
pub use app::TemplateApp;

mod objects;

pub use objects::FileObject;
pub use objects::Funds;

