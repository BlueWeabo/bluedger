#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(clippy::print_stderr)] // Using eprintln! for error logging in CLI/server context

#[cfg(not(target_arch = "wasm32"))]
use std::{
    fs::{read_to_string, write},
    process::Command,
};

#[cfg(not(target_arch = "wasm32"))]
use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
#[cfg(not(target_arch = "wasm32"))]
use bluedger::{FileObject, Funds, YearlyGraphPoints};
#[cfg(not(target_arch = "wasm32"))]
use chrono::Datelike as _;
#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Utc};

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
        .route("/api/assets", get(total_assets))
        .route("/api/expenses_plot", get(expenses_plot))
        .route("/api/income_plot", get(income_plot))
        .route("/api/assets_plot", get(assets_plot))
        .route("/api/liabilities_plot", get(liabilities_plot))
        .route("/api/equity_plot", get(equity_plot));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:21000")
        .await
        .expect("Failed to bind to port 21000");
    axum::serve(listener, app).await.expect("Failed to serve");
}

#[cfg(not(target_arch = "wasm32"))]
async fn get_file(Json(payload): Json<FileObject>) -> (StatusCode, Json<FileObject>) {
    let year = payload.year;
    let month = payload.month;
    let contents = if year == 0 && month == 0 {
        read_to_string("/home/blueweabo/.config/bluedger/master.dat")
            .expect("Failed to read master.dat")
    } else {
        read_to_string(format!(
            "/home/blueweabo/.config/bluedger/{year}/{month}.dat",
        ))
        .expect("Failed to read monthly file")
    };
    let file = FileObject {
        year,
        month,
        contents,
    };
    (StatusCode::OK, Json(file))
}

#[cfg(not(target_arch = "wasm32"))]
async fn update_file(Json(payload): Json<FileObject>) -> StatusCode {
    let year = payload.year;
    let month = payload.month;
    let contents = payload.contents;

    if year == 0 && month == 0 {
        write("/home/blueweabo/.config/bluedger/master.dat", contents)
            .expect("Failed to write master.dat");
    } else {
        write(
            format!("/home/blueweabo/.config/bluedger/{year}/{month}.dat",),
            contents,
        )
        .expect("Failed to write monthly file");
    }
    StatusCode::OK
}

#[cfg(not(target_arch = "wasm32"))]
async fn total_assets() -> (StatusCode, Json<Funds>) {
    let output = match Command::new("ledger")
        .arg("-f")
        .arg("/home/blueweabo/.config/bluedger/master.dat")
        .arg("bal")
        .arg("Assets")
        .arg("-n")
        .arg("-F")
        .arg("%(strip(display_total))")
        .output()
    {
        Ok(o) => {
            if !o.status.success() {
                eprintln!(
                    "command error while getting current assets: {}",
                    String::from_utf8(o.stderr).unwrap_or_default()
                );
            }
            match String::from_utf8(o.stdout) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("Error running command while getting assets: {e}");
                    String::new()
                }
            }
        }
        Err(e) => {
            eprintln!("Error running command while getting assets: {e}");
            String::new()
        }
    };
    let iter = output.split('\n');
    let mut count = 0;
    let mut amounts: Vec<f64> = Vec::new();
    let mut curr: Vec<String> = Vec::new();
    for val in iter {
        count += 1;
        let mut split_val = val.split_whitespace();
        let value_str = match split_val.next() {
            None => "0 XXX",
            Some(str) => str,
        };
        let value_parsed = match value_str.replace(',', ".").parse() {
            Err(e) => {
                eprintln!("Error parsing first value: {e}");
                0.0
            }
            Ok(num) => num,
        };
        amounts.push(value_parsed);
        let curr_str = match split_val.next() {
            None => "0.0 XXX",
            Some(str) => str,
        };
        curr.push(curr_str.to_owned());
    }
    let funds = Funds {
        amounts,
        currencies: curr,
        size: count,
    };
    (StatusCode::OK, Json(funds))
}

#[cfg(not(target_arch = "wasm32"))]
async fn last_month_expenses() -> (StatusCode, Json<Funds>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month();
    let year = date.year();
    let begin_date = if month == 1 {
        format!("{year}/12/01", year = year - 1)
    } else {
        format!("{year}/{month}/01", month = month - 1)
    };
    let end_date = if month == 1 {
        format!("{year}/12/31", year = year - 1)
    } else {
        let end_day = date
            .date_naive()
            .with_day(1)
            .expect("Failed to set day to 1")
            .with_month(month - 1)
            .expect("Failed to set month")
            .num_days_in_month();
        format!(
            "{year}/{month}/{end_day}",
            year = year,
            month = month - 1,
            end_day = end_day
        )
    };
    let output = ledger_balance(begin_date, end_date, "Expenses".to_owned());
    let iter = output.split('\n');
    let mut count = 0;
    let mut amounts: Vec<f64> = Vec::new();
    let mut curr: Vec<String> = Vec::new();
    for val in iter {
        count += 1;
        let mut split_val = val.split_whitespace();
        let value_str = match split_val.next() {
            None => "0 XXX",
            Some(str) => str,
        };
        let value_parsed = match value_str.replace(',', ".").parse() {
            Err(e) => {
                eprintln!("Error parsing first value: {e}");
                0.0
            }
            Ok(num) => num,
        };
        amounts.push(value_parsed);
        let curr_str = match split_val.next() {
            None => "0.0 XXX",
            Some(str) => str,
        };
        curr.push(curr_str.to_owned());
    }
    let funds = Funds {
        amounts,
        currencies: curr,
        size: count,
    };
    (StatusCode::OK, Json(funds))
}

#[cfg(not(target_arch = "wasm32"))]
async fn current_expenses() -> (StatusCode, Json<Funds>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month();
    let year = date.year();
    let begin_date = format!("{year}/{month}/01");
    let end_day = date.date_naive().num_days_in_month();
    let end_date = format!("{year}/{month}/{end_day}");
    let output = ledger_balance(begin_date, end_date, "Expenses".to_owned());
    let iter = output.split('\n');
    let mut count = 0;
    let mut amounts: Vec<f64> = Vec::new();
    let mut curr: Vec<String> = Vec::new();
    for val in iter {
        count += 1;
        let mut split_val = val.split_whitespace();
        let value_str = match split_val.next() {
            None => "0 XXX",
            Some(str) => str,
        };
        let value_parsed = match value_str.replace(',', ".").parse() {
            Err(e) => {
                eprintln!("Error parsing first value: {e}");
                0.0
            }
            Ok(num) => num,
        };
        amounts.push(value_parsed);
        let curr_str = match split_val.next() {
            None => "0.0 XXX",
            Some(str) => str,
        };
        curr.push(curr_str.to_owned());
    }
    let funds = Funds {
        amounts,
        currencies: curr,
        size: count,
    };
    (StatusCode::OK, Json(funds))
}

#[cfg(not(target_arch = "wasm32"))]
async fn expenses_plot() -> (StatusCode, Json<YearlyGraphPoints>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month().cast_signed();
    let year = date.year();
    let mut expense_records = YearlyGraphPoints { points: Vec::new() };
    for val in 0..13 {
        let year_to_check = if month - 12 + val <= 0 {
            year - 1
        } else {
            year
        };
        let month_to_check = if month - 12 + val <= 0 {
            month + val
        } else {
            month - 12 + val
        };
        let end_day = Utc::now()
            .date_naive()
            .with_day(1)
            .expect("Failed to set day to 1")
            .with_month(month_to_check as u32)
            .expect("Failed to set month")
            .num_days_in_month();
        let begin_date = format!("{year_to_check}/{month_to_check}/01");
        let end_date = format!("{year_to_check}/{month_to_check}/{end_day}");
        let output = ledger_balance_converted(begin_date, end_date, "Expenses".to_owned());
        expense_records.points.push([val as f64, output]);
    }
    (StatusCode::OK, Json(expense_records))
}

#[cfg(not(target_arch = "wasm32"))]
async fn income_plot() -> (StatusCode, Json<YearlyGraphPoints>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month().cast_signed();
    let year = date.year();
    let mut expense_records = YearlyGraphPoints { points: Vec::new() };
    for val in 0..13 {
        let year_to_check = if month - 12 + val <= 0 {
            year - 1
        } else {
            year
        };
        let month_to_check = if month - 12 + val <= 0 {
            month + val
        } else {
            month - 12 + val
        };
        let end_day = Utc::now()
            .date_naive()
            .with_day(1)
            .expect("Failed to set day to 1")
            .with_month(month_to_check as u32)
            .expect("Failed to set month")
            .num_days_in_month();
        let begin_date = format!("{year_to_check}/{month_to_check}/01");
        let end_date = format!("{year_to_check}/{month_to_check}/{end_day}");
        let output = ledger_balance_converted(begin_date, end_date, "Income".to_owned());
        // Multiply by -1 to get a positive number
        expense_records.points.push([val as f64, -output]);
    }
    (StatusCode::OK, Json(expense_records))
}

#[cfg(not(target_arch = "wasm32"))]
async fn assets_plot() -> (StatusCode, Json<YearlyGraphPoints>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month().cast_signed();
    let year = date.year();
    let mut assets_records = YearlyGraphPoints { points: Vec::new() };
    for val in 0..13 {
        let year_to_check = if month - 12 + val <= 0 {
            year - 1
        } else {
            year
        };
        let month_to_check = if month - 12 + val <= 0 {
            month + val
        } else {
            month - 12 + val
        };
        let end_day = Utc::now()
            .date_naive()
            .with_day(1)
            .expect("Failed to set day to 1")
            .with_month(month_to_check as u32)
            .expect("Failed to set month")
            .num_days_in_month();
        let end_date = format!("{year_to_check}/{month_to_check}/{end_day}");
        let output =
            ledger_balance_converted("2000/01/01".to_owned(), end_date, "Assets".to_owned());
        assets_records.points.push([val as f64, output]);
    }
    (StatusCode::OK, Json(assets_records))
}

#[cfg(not(target_arch = "wasm32"))]
async fn liabilities_plot() -> (StatusCode, Json<YearlyGraphPoints>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month().cast_signed();
    let year = date.year();
    let mut assets_records = YearlyGraphPoints { points: Vec::new() };
    for val in 0..13 {
        let year_to_check = if month - 12 + val <= 0 {
            year - 1
        } else {
            year
        };
        let month_to_check = if month - 12 + val <= 0 {
            month + val
        } else {
            month - 12 + val
        };
        let end_day = Utc::now()
            .date_naive()
            .with_day(1)
            .expect("Failed to set day to 1")
            .with_month(month_to_check as u32)
            .expect("Failed to set month")
            .num_days_in_month();
        let end_date = format!("{year_to_check}/{month_to_check}/{end_day}");
        let output =
            ledger_balance_converted("2000/01/01".to_owned(), end_date, "Liabilities".to_owned());
        // Multiply by -1 so that it is shown as positive on the plot
        assets_records.points.push([val as f64, -output]);
    }
    (StatusCode::OK, Json(assets_records))
}

#[cfg(not(target_arch = "wasm32"))]
async fn equity_plot() -> (StatusCode, Json<YearlyGraphPoints>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month().cast_signed();
    let year = date.year();
    let mut assets_records = YearlyGraphPoints { points: Vec::new() };
    for val in 0..13 {
        let year_to_check = if month - 12 + val <= 0 {
            year - 1
        } else {
            year
        };
        let month_to_check = if month - 12 + val <= 0 {
            month + val
        } else {
            month - 12 + val
        };
        let end_day = Utc::now()
            .date_naive()
            .with_day(1)
            .expect("Failed to set day to 1")
            .with_month(month_to_check as u32)
            .expect("Failed to set month")
            .num_days_in_month();
        let end_date = format!("{year_to_check}/{month_to_check}/{end_day}");
        let output =
            ledger_balance_converted("2000/01/01".to_owned(), end_date, "Equity".to_owned());
        assets_records.points.push([val as f64, output]);
    }
    (StatusCode::OK, Json(assets_records))
}

#[cfg(not(target_arch = "wasm32"))]
fn ledger_balance_converted(begin_date: String, end_date: String, balances: String) -> f64 {
    match Command::new("ledger")
        .arg("-f")
        .arg("/home/blueweabo/.config/bluedger/master.dat")
        .arg("bal")
        .arg(balances)
        .arg("-n")
        .arg("-X")
        .arg("EUR")
        .arg("-F")
        .arg("%(quantity(display_total))")
        .arg("--begin")
        .arg(begin_date)
        .arg("--end")
        .arg(end_date)
        .output()
    {
        Ok(o) => {
            if !o.status.success() {
                eprintln!(
                    "command error while getting balance: {}",
                    String::from_utf8(o.stderr).unwrap_or_default()
                );
            }
            match String::from_utf8(o.stdout) {
                Ok(o) => {
                    if o.is_empty() {
                        0.0
                    } else {
                        match o.parse::<f64>() {
                            Ok(v) => v,
                            Err(rv) => {
                                eprintln!("Error converting to float: {rv}");
                                0.0
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error running command: {e}");
                    0.0
                }
            }
        }
        Err(e) => {
            eprintln!("Error running command: {e}");
            0.0
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn ledger_balance(begin_date: String, end_date: String, balances: String) -> String {
    match Command::new("ledger")
        .arg("-f")
        .arg("/home/blueweabo/.config/bluedger/master.dat")
        .arg("bal")
        .arg(balances)
        .arg("-n")
        .arg("-F")
        .arg("%(strip(display_total))")
        .arg("--begin")
        .arg(begin_date)
        .arg("--end")
        .arg(end_date)
        .output()
    {
        Ok(o) => {
            if !o.status.success() {
                eprintln!(
                    "command error while getting balance: {}",
                    String::from_utf8(o.stderr).unwrap_or_default()
                );
            }
            match String::from_utf8(o.stdout) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("Error converting to String: {e}");
                    String::new()
                }
            }
        }
        Err(e) => {
            eprintln!("Error running command: {e}");
            String::new()
        }
    }
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
