use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct FileObject {
    pub year: u64,
    pub month: u64,
    pub contents: String,
}

#[derive(Deserialize, Serialize)]
pub struct Funds {
    pub amounts: Vec<f64>,
    pub currencies: Vec<String>,
    pub size: u64,
}

#[derive(Deserialize, Serialize)]
pub struct YearlyGraphPoints {
    pub points: Vec<[f64; 2]>,
}
