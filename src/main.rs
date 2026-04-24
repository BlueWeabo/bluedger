#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};

// When compiling for server api
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    env_logger::init();

    let app = Router::new()
        .route("/api/get", post(get_file))
        .route("/api/update", post(update_file));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:21000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
async fn get_file(Json(payload): Json<FileObject>,) -> (StatusCode, Json<FileObject>) {
    use std::{fmt::format, fs::read_to_string};
    let year = payload.year;
    let month = payload.month;
    let contents = read_to_string(format(format_args!("/home/blueweabo/.config/bluedger/{}/{}.dat", year, month))).unwrap();
    let file = FileObject {
        year,
        month,
        contents,
    };
    (StatusCode::OK, Json(file))
}

#[cfg(not(target_arch = "wasm32"))]
async fn update_file(Json(payload): Json<FileObject>,) -> (StatusCode) {
    use std::{fmt::format, fs::{write}};
    let year = payload.year;
    let month = payload.month;
    let contents = payload.contents;

    let _ = write(format(format_args!("/home/blueweabo/.config/bluedger/{}/{}.dat", year, month)), contents);
    (StatusCode::OK)
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize)]
#[derive(Deserialize)]
struct FileObject {
    year: u64,
    month: u64,
    contents: String,
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(bluedger::TemplateApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
