use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Add, Sub, Mul, Div};
use std::fmt;
use std::fs;
use chrono::{Local, Timelike};

// Simple fixed-point type with 2 decimal places (scale factor of 100).
// For example, 1.23 is stored as 123.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]

pub struct FixedPoint(i64);
impl FixedPoint {
    const SCALE: i64 = 100; // 10^2 for 2 decimal places
    
    // Constructor from a float (e.g., FixedPoint::from_float(1.23))
    pub fn from_float(value: f64) -> Self {
        Self((value * Self::SCALE as f64).round() as i64)
    }
    
    // Constructor from an integer (e.g., FixedPoint::from_int(123) for 1.23)
    pub fn from_int(value: i64) -> Self {
        Self(value)
    }
    
    // Convert back to float for display or calculations
    pub fn to_float(self) -> f64 {
        self.0 as f64 / Self::SCALE as f64
    }
    
    // Get the raw scaled value
    pub fn raw(self) -> i64 {
        self.0
    }
}

impl Add for FixedPoint {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for FixedPoint {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Mul for FixedPoint {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        // Scale down after multiplication to maintain precision
        Self((self.0 * other.0) / Self::SCALE)
    }
}

impl Div for FixedPoint {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        // Scale up before division
        Self((self.0 * Self::SCALE) / other.0)
    }
}

impl fmt::Display for FixedPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.2}", self.to_float())
    }
}

fn deserialize_fixed_point<'de, D>(deserializer: D) -> Result<FixedPoint, D::Error> where D: serde::Deserializer<'de>,
{
    let value: f64 = Deserialize::deserialize(deserializer)?;
    Ok(FixedPoint::from_float(value))
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct Order {
    amount: u64, // Highest seen: 1186070
    #[serde(deserialize_with = "deserialize_fixed_point")]
    pricePerUnit: FixedPoint,
    orders: u32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct QuickStatus {
    productId: String, 
    sellPrice: f64, // MANDATORY
    sellVolume: u64, // Highest seen: 1292216
    sellMovingWeek: u64, // Highest seen: 188604293
    sellOrders: u32, // Highest seen: 202
    buyPrice: f64, // float IS MANDATORY
    buyVolume: u64, // Highest seen: 11766801
    buyMovingWeek: u64, // Highest seen: 9205352
    buyOrders: u32, // Highest seen: 270
}

#[derive(Deserialize, Serialize)]
struct Product {
    product_id: String,
    sell_summary: Vec<Order>,
    buy_summary: Vec<Order>,
    quick_status: QuickStatus,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct BazaarResponse {
    success: bool,
    lastUpdated: u64,
    products: HashMap<String, Product>,
}

fn get_and_dump() -> Result<(), Box<dyn std::error::Error>> {
    let response: BazaarResponse =
    reqwest::blocking::get("https://api.hypixel.net/v2/skyblock/bazaar")?.json()?;
    
    println!("Success: {}", response.success);
    println!("Last updated: {}", response.lastUpdated);
    println!("Number of products: {}", response.products.len());
    
    // Create raw directory if it doesn't exist
    fs::create_dir_all("raw")?;
    
    // Generate filename with YYYYMMDD_<seconds-from-midnight> format
    let now: chrono::DateTime<Local> = Local::now();
    let date_str: String = now.format("%Y%m%d").to_string();
    let seconds_from_midnight: u32 = (now.hour() * 3600)
    + (now.minute() * 60)
    + now.second();
    let filename: String = format!("raw/{}_{:05}.json", date_str, seconds_from_midnight);
    
    // Serialize response to JSON and write to file
    let json: String = serde_json::to_string_pretty(&response)?;
    fs::write(&filename, json)?;
    
    println!("Response saved to: {}", filename);
    
    Ok(())
}

fn generate_csv() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    get_and_dump()?;
    generate_csv()?;
    Ok(())
}

