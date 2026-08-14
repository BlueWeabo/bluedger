#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
use std::{
    fmt::format,
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
        .route("/api/assets", get(total_assets))
        .route("/api/expenses_plot", get(expenses_plot))
        .route("/api/income_plot", get(income_plot))
        .route("/api/assets_plot", get(assets_plot))
        .route("/api/liabilities_plot", get(liabilities_plot))
        .route("/api/equity_plot", get(equity_plot));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:21000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
async fn get_file(Json(payload): Json<FileObject>) -> (StatusCode, Json<FileObject>) {
    let year = payload.year;
    let month = payload.month;
    let contents: String;
    if year == 0 && month == 0 {
        contents = read_to_string("/home/blueweabo/.config/bluedger/master.dat").unwrap();
    } else {
        contents = read_to_string(format(format_args!(
            "/home/blueweabo/.config/bluedger/{}/{}.dat",
            year, month
        )))
        .unwrap();
    }
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
        let _ = write("/home/blueweabo/.config/bluedger/master.dat", contents);
    } else {
        let _ = write(
            format(format_args!(
                "/home/blueweabo/.config/bluedger/{}/{}.dat",
                year, month
            )),
            contents,
        );
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
                println!(
                    "command error while getting current assets: {}",
                    String::from_utf8(o.stderr).unwrap()
                );
            }
            match String::from_utf8(o.stdout) {
                Ok(o) => o,
                Err(e) => {
                    println!("Error running command while getting assets: {}", e);
                    "".to_owned()
                }
            }
        }
        Err(e) => {
            println!("Error running command while getting assets: {}", e);
            "".to_owned()
        }
    };
    let iter = output.split("\n");
    let mut count = 0;
    let mut amounts: Vec<f64> = Vec::new();
    let mut curr: Vec<String> = Vec::new();
    for val in iter {
        count += 1;
        let mut split_val = val.split(" ");
        let value_str = match split_val.next() {
            None => "0 XXX",
            Some(str) => str,
        };
        let value_parsed = match value_str.replace(",", ".").parse() {
            Err(e) => {
                println!("Error parsing first value: {}", e);
                0.0
            }
            Ok(num) => num,
        };
        amounts.push(value_parsed);
        let curr_str = match split_val.next() {
            None => "0.0 XXX",
            Some(str) => str,
        };
        curr.push(curr_str.to_string());
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
    let begin_date: String;
    let end_date: String;
    if month == 1 {
        begin_date = format!("{}/{}/01", year - 1, 12);
        end_date = format!("{}/{}/31", year - 1, 12);
    } else {
        begin_date = format!("{}/{}/01", year, month - 1);
        let end_day = date
            .date_naive()
            .with_day(1)
            .unwrap()
            .with_month(month - 1)
            .unwrap()
            .num_days_in_month();
        end_date = format!("{}/{}/{}", year, month - 1, end_day);
    }
    let output = ledger_balance(begin_date, end_date, "Expenses".to_owned()).await;
    let iter = output.split("\n");
    let mut count = 0;
    let mut amounts: Vec<f64> = Vec::new();
    let mut curr: Vec<String> = Vec::new();
    for val in iter {
        count += 1;
        let mut split_val = val.split(" ");
        let value_str = match split_val.next() {
            None => "0 XXX",
            Some(str) => str,
        };
        let value_parsed = match value_str.replace(",", ".").parse() {
            Err(e) => {
                println!("Error parsing first value: {}", e);
                0.0
            }
            Ok(num) => num,
        };
        amounts.push(value_parsed);
        let curr_str = match split_val.next() {
            None => "0.0 XXX",
            Some(str) => str,
        };
        curr.push(curr_str.to_string());
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
    let begin_date = format!("{}/{}/01", year, month);
    let end_day = date.date_naive().num_days_in_month();
    let end_date = format!("{}/{}/{}", year, month, end_day);
    let output = ledger_balance(begin_date, end_date, "Expenses".to_owned()).await;
    let iter = output.split("\n");
    let mut count = 0;
    let mut amounts: Vec<f64> = Vec::new();
    let mut curr: Vec<String> = Vec::new();
    for val in iter {
        count += 1;
        let mut split_val = val.split(" ");
        let value_str = match split_val.next() {
            None => "0 XXX",
            Some(str) => str,
        };
        let value_parsed = match value_str.replace(",", ".").parse() {
            Err(e) => {
                println!("Error parsing first value: {}", e);
                0.0
            }
            Ok(num) => num,
        };
        amounts.push(value_parsed);
        let curr_str = match split_val.next() {
            None => "0.0 XXX",
            Some(str) => str,
        };
        curr.push(curr_str.to_string());
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
    let month = date.month() as i32;
    let year = date.year();
    let mut expense_records = YearlyGraphPoints { points: Vec::new() };
    for val in 0..13 {
        let begin_date: String;
        let end_date: String;
        let year_to_check: i32;
        let month_to_check = if month - 12 + val <= 0 {
            year_to_check = year - 1;
            month + val
        } else {
            year_to_check = year;
            month - 12 + val
        };
        let date: DateTime<Utc> = Utc::now();
        begin_date = format!("{}/{}/01", year_to_check, month_to_check);
        let end_day = date
            .date_naive()
            .with_day(1)
            .unwrap()
            .with_month(month_to_check as u32)
            .unwrap()
            .num_days_in_month();
        end_date = format!("{}/{}/{}", year_to_check, month_to_check, end_day);
        let output = ledger_balance_converted(begin_date, end_date, "Expenses".to_owned()).await;
        expense_records.points.push([val as f64, output]);
    }
    (StatusCode::OK, Json(expense_records))
}

#[cfg(not(target_arch = "wasm32"))]
async fn income_plot() -> (StatusCode, Json<YearlyGraphPoints>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month() as i32;
    let year = date.year();
    let mut expense_records = YearlyGraphPoints { points: Vec::new() };
    for val in 0..13 {
        let begin_date: String;
        let end_date: String;
        let year_to_check: i32;
        let month_to_check = if month - 12 + val <= 0 {
            year_to_check = year - 1;
            month + val
        } else {
            year_to_check = year;
            month - 12 + val
        };
        let date: DateTime<Utc> = Utc::now();
        begin_date = format!("{}/{}/01", year_to_check, month_to_check);
        let end_day = date
            .date_naive()
            .with_day(1)
            .unwrap()
            .with_month(month_to_check as u32)
            .unwrap()
            .num_days_in_month();
        end_date = format!("{}/{}/{}", year_to_check, month_to_check, end_day);
        let output = ledger_balance_converted(begin_date, end_date, "Income".to_owned()).await;
        // Multiple by -1 to get a positive number
        expense_records.points.push([val as f64, output * (-1.0)]);
    }
    (StatusCode::OK, Json(expense_records))
}

#[cfg(not(target_arch = "wasm32"))]
async fn assets_plot() -> (StatusCode, Json<YearlyGraphPoints>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month() as i32;
    let year = date.year();
    let mut assets_records = YearlyGraphPoints { points: Vec::new() };
    for val in 0..13 {
        let end_date: String;
        let year_to_check: i32;
        let month_to_check = if month - 12 + val <= 0 {
            year_to_check = year - 1;
            month + val
        } else {
            year_to_check = year;
            month - 12 + val
        };
        let date: DateTime<Utc> = Utc::now();
        let end_day = date
            .date_naive()
            .with_day(1)
            .unwrap()
            .with_month(month_to_check as u32)
            .unwrap()
            .num_days_in_month();
        end_date = format!("{}/{}/{}", year_to_check, month_to_check, end_day);
        let output =
            ledger_balance_converted("2000/01/01".to_owned(), end_date, "Assets".to_owned()).await;
        assets_records.points.push([val as f64, output]);
    }
    (StatusCode::OK, Json(assets_records))
}

#[cfg(not(target_arch = "wasm32"))]
async fn liabilities_plot() -> (StatusCode, Json<YearlyGraphPoints>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month() as i32;
    let year = date.year();
    let mut assets_records = YearlyGraphPoints { points: Vec::new() };
    for val in 0..13 {
        let end_date: String;
        let year_to_check: i32;
        let month_to_check = if month - 12 + val <= 0 {
            year_to_check = year - 1;
            month + val
        } else {
            year_to_check = year;
            month - 12 + val
        };
        let date: DateTime<Utc> = Utc::now();
        let end_day = date
            .date_naive()
            .with_day(1)
            .unwrap()
            .with_month(month_to_check as u32)
            .unwrap()
            .num_days_in_month();
        end_date = format!("{}/{}/{}", year_to_check, month_to_check, end_day);
        let output =
            ledger_balance_converted("2000/01/01".to_owned(), end_date, "Liabilities".to_owned())
                .await;
        // Multiply by -1 so that it is shown as positive on the plot
        assets_records.points.push([val as f64, output * (-1.0)]);
    }
    (StatusCode::OK, Json(assets_records))
}

#[cfg(not(target_arch = "wasm32"))]
async fn equity_plot() -> (StatusCode, Json<YearlyGraphPoints>) {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month() as i32;
    let year = date.year();
    let mut assets_records = YearlyGraphPoints { points: Vec::new() };
    for val in 0..13 {
        let end_date: String;
        let year_to_check: i32;
        let month_to_check = if month - 12 + val <= 0 {
            year_to_check = year - 1;
            month + val
        } else {
            year_to_check = year;
            month - 12 + val
        };
        let date: DateTime<Utc> = Utc::now();
        let end_day = date
            .date_naive()
            .with_day(1)
            .unwrap()
            .with_month(month_to_check as u32)
            .unwrap()
            .num_days_in_month();
        end_date = format!("{}/{}/{}", year_to_check, month_to_check, end_day);
        let output =
            ledger_balance_converted("2000/01/01".to_owned(), end_date, "Equity".to_owned()).await;
        assets_records.points.push([val as f64, output]);
    }
    (StatusCode::OK, Json(assets_records))
}

#[cfg(not(target_arch = "wasm32"))]
async fn ledger_balance_converted(begin_date: String, end_date: String, balances: String) -> f64 {
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
                println!(
                    "command error while getting current assets: {}",
                    String::from_utf8(o.stderr).unwrap()
                );
            }
            match String::from_utf8(o.stdout) {
                Ok(o) => {
                    if o == "" {
                        0.0
                    } else {
                        match o.parse::<f64>() {
                            Ok(v) => v,
                            Err(rv) => {
                                println!("Error converting to float: {}", rv);
                                0.0
                            }
                        }
                    }
                }

                Err(e) => {
                    println!("Error running command while getting assets: {}", e);
                    0.0
                }
            }
        }
        Err(e) => {
            println!("Error running command while getting assets: {}", e);
            0.0
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn ledger_balance(begin_date: String, end_date: String, balances: String) -> String {
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
                println!(
                    "command error while getting current assets: {}",
                    String::from_utf8(o.stderr).unwrap()
                );
            }
            match String::from_utf8(o.stdout) {
                Ok(o) => o,
                Err(e) => {
                    println!(
                        "Error converting to String while getting current expenses: {}",
                        e
                    );
                    "".to_owned()
                }
            }
        }
        Err(e) => {
            println!(
                "Error running command while getting current expenses: {}",
                e
            );
            "".to_owned()
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
