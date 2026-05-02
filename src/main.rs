#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
use std::{fmt::format, fs::{read_to_string, write}, process::Command};

#[cfg(not(target_arch = "wasm32"))]
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
#[cfg(not(target_arch = "wasm32"))]
use bluedger::{FileObject, Funds};
#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Datelike, Utc};

// When compiling for server api
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    env_logger::init();

    let app = Router::new()
        .route("/api/get", post(get_file))
        .route("/api/update", post(update_file))
        .route("/api/last_month_expenses", get(last_month_expenses))
        .route("/api/current_expenses", get(current_expenses))
        .route("/api/assets", get(total_assets));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:21000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
async fn get_file(Json(payload): Json<FileObject>,) -> (StatusCode, Json<FileObject>) {
    let year = payload.year;
    let month = payload.month;
    let contents: String;
    if year == 0 && month == 0 {
        contents = read_to_string("/home/blueweabo/.config/bluedger/master.dat").unwrap();
    } else {
        contents = read_to_string(format(format_args!("/home/blueweabo/.config/bluedger/{}/{}.dat", year, month))).unwrap();
    }
    let file = FileObject {
        year,
        month,
        contents,
    };
    (StatusCode::OK, Json(file))
}

#[cfg(not(target_arch = "wasm32"))]
async fn update_file(Json(payload): Json<FileObject>,) -> StatusCode {
    let year = payload.year;
    let month = payload.month;
    let contents = payload.contents;

    if year == 0 && month == 0 {
        let _ = write("/home/blueweabo/.config/bluedger/master.dat", contents);
    } else {
        let _ = write(format(format_args!("/home/blueweabo/.config/bluedger/{}/{}.dat", year, month)), contents);
    }
    StatusCode::OK
}

#[cfg(not(target_arch = "wasm32"))]
async fn total_assets() -> (StatusCode, Json<Funds>) {
    let ledger = Command::new("ledger")
        .arg("-f")
        .arg("/home/blueweabo/.config/bluedger/master.dat")
        .arg("bal")
        .arg("Assets")
        .arg("-n")
        .arg("-F")
        .arg("%(strip(display_total))")
        .output()
        .expect("There was an error");
    let output = String::from_utf8( ledger.stdout).unwrap();
    let iter = output.split("\n");
    let mut count = 0;
    let mut amounts :Vec<f64> = Vec::new();
    let mut curr : Vec<String> = Vec::new();
    for val in iter {
        count += 1;
        let mut split_val = val.split(" ");
        amounts.push(split_val.next().unwrap().replace(",", ".").parse().unwrap());
        curr.push(split_val.next().unwrap().to_string());
    }
    let funds = Funds {
        amounts,
        currencies : curr,
        size : count,
    };
    (StatusCode::OK, Json(funds))
}

#[cfg(not(target_arch = "wasm32"))]
async fn last_month_expenses() -> (StatusCode, Json<Funds>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month();
    let year = date.year();
    let begin_date: String;
    let end_date: String;
    if month == 1 {
        begin_date = format!("{}/{}/01", year - 1, 12);
        end_date = format!("{}/{}/31", year - 1, 12);
    } else {
        begin_date = format!("{}/{}/01", year, month - 1);
        let end_day = date.date_naive().with_month(month - 1).unwrap().num_days_in_month();
        end_date = format!("{}/{}/{}", year, month - 1, end_day);
    }
    let ledger = Command::new("ledger")
        .arg("-f")
        .arg("/home/blueweabo/.config/bluedger/master.dat")
        .arg("bal")
        .arg("Expenses")
        .arg("-n")
        .arg("-F")
        .arg("%(strip(display_total))")
        .arg("--begin")
        .arg(begin_date)
        .arg("--end")
        .arg(end_date)
        .output()
        .expect("There was an error");
    let output = String::from_utf8( ledger.stdout).unwrap();
    println!("output: {}", output);
    let iter = output.split("\n");
    let mut count = 0;
    let mut amounts :Vec<f64> = Vec::new();
    let mut curr : Vec<String> = Vec::new();
    for val in iter {
        count += 1;
        let mut split_val = val.split(" ");
        amounts.push(split_val.next().unwrap().replace(",", ".").parse().unwrap());
        curr.push(split_val.next().unwrap().to_string());
    }
    let funds = Funds {
        amounts,
        currencies : curr,
        size : count,
    };
    (StatusCode::OK, Json(funds))
}

#[cfg(not(target_arch = "wasm32"))]
async fn current_expenses() -> (StatusCode, Json<Funds>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month();
    let year = date.year();
    let begin_date = format!("{}/{}/01", year, month);
    let end_day = date.date_naive().num_days_in_month();
    let end_date = format!("{}/{}/{}", year, month, end_day);
    let ledger = Command::new("ledger")
        .arg("-f")
        .arg("/home/blueweabo/.config/bluedger/master.dat")
        .arg("bal")
        .arg("Expenses")
        .arg("-n")
        .arg("-F")
        .arg("%(strip(display_total))")
        .arg("--begin")
        .arg(begin_date)
        .arg("--end")
        .arg(end_date)
        .output()
        .expect("There was an error");
    let output = String::from_utf8( ledger.stdout).unwrap();
    println!("output: {}", output);
    let iter = output.split("\n");
    let mut count = 0;
    let mut amounts :Vec<f64> = Vec::new();
    let mut curr : Vec<String> = Vec::new();
    for val in iter {
        count += 1;
        let mut split_val = val.split(" ");
        amounts.push(split_val.next().unwrap().replace(",", ".").parse().unwrap());
        curr.push(split_val.next().unwrap().to_string());
    }
    let funds = Funds {
        amounts,
        currencies : curr,
        size : count,
    };
    (StatusCode::OK, Json(funds))
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
