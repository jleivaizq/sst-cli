use chrono::{NaiveDate,};
use chrono::prelude::*;
use yahoo_finance_api as yahoo;
use yahoo::{Quote, YahooError};
use log::{info, error};

#[derive(Debug)]
pub struct StocKpi {
    pub from: NaiveDate,
    pub symbol: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub price_diff: Option<(f64, f64)>,
    pub sma: Option<Vec<f64>>,
    pub last_quote: Option<Quote>,
}

impl StocKpi {
    pub fn new(symbol: &str, from_src: NaiveDate) -> Self {
        StocKpi {
            from: from_src,
            symbol: String::from(symbol),
            min: None,
            max: None,
            price_diff: None,
            sma: None,
            last_quote: None
        } 
    } 

    fn retrieve_quotes(symbol: &str, from: &NaiveDate) -> Result<Vec<Quote>, YahooError> {
        info!("Downloading values for symbol {} from {}", symbol, from);
        let provider = yahoo::YahooConnector::new();
        let start = Utc.ymd(from.year(), from.month(), from.day()).and_hms_milli(0, 0, 0, 0);
        provider.get_quote_history(&symbol, start, Utc::now())?.quotes()
    }

    fn min(series: &[f64]) -> Option<f64> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MAX, |acc, q| acc.min(*q)))
        }
    }

    fn max(series: &[f64]) -> Option<f64> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MIN, |acc, q| acc.max(*q)))
        }
    }

    fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
        if series.is_empty() {
            None
        } else {
            Some(series.windows(n)
                       .map(|w| w.iter().sum::<f64>() / w.len() as f64)
                       .collect())
        }
    }

    fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
        if series.is_empty() {
            None
        } else {
            let first_price = series.first().unwrap();
            let last_price = series.last().unwrap();
            let price_diff = (first_price - last_price).abs();
            Some((price_diff / first_price, price_diff))
        }
    }

    pub fn last_sma(self: &StocKpi) -> Option<f64> {
        if let Some(sma) = &self.sma {
            Some(sma.last().unwrap().clone())
        } else {
            None
        }
    }

    pub fn calculate(self: &mut StocKpi, sma_window: usize) { 
        match StocKpi::retrieve_quotes(&self.symbol, &self.from) {
            Ok(quotes) => {
                let closes: Vec<f64> = quotes.iter().map(|q| q.adjclose as f64).collect();
                self.min = StocKpi::min(&closes[..]);
                self.max = StocKpi::max(&closes[..]);
                self.sma = StocKpi::n_window_sma(sma_window, &closes[..]);
                self.price_diff = StocKpi::price_diff(&closes[..]);
                self.last_quote = Some(quotes.last().unwrap().clone());
            },
            Err(error) => {
                error!("Could not calculate performance metrics for {} from {}. Error: {}", 
                    self.symbol, self.from, error);
            }
        }
    }

}