use colored::Colorize;
use eyre::{Context, ContextCompat};
use std::fmt;
use zksync_ethers_rs::{core::utils::format_units, types::U256};

#[derive(Debug, Clone)]
pub struct GasTracker {
    gas: Vec<U256>,
    fees: Vec<U256>,
    gas_prices: Vec<U256>,
    txs_per_run: Vec<u64>,
    token_symbol: String,
    token_decimals: i32,
}

fn sum_vec_u256(vec: &[U256]) -> U256 {
    let mut total = U256::zero();
    for v in vec {
        total += *v;
    }
    total
}

impl GasTracker {
    pub fn new() -> Self {
        GasTracker {
            gas: Vec::new(),
            fees: Vec::new(),
            gas_prices: Vec::new(),
            txs_per_run: Vec::new(),
            token_symbol: "None".to_owned(),
            token_decimals: 18,
        }
    }

    pub fn set_token_symbol(mut self, token_symbol: String) -> Self {
        self.token_symbol = token_symbol;
        self
    }

    pub fn set_token_decimals(mut self, token_decimals: i32) -> Self {
        self.token_decimals = token_decimals;
        self
    }

    pub fn add_run(&mut self, gas: U256, fee: U256, gas_price: U256, txs_per_run: u64) {
        self.gas.push(gas);
        self.fees.push(fee);
        self.gas_prices.push(gas_price);
        self.txs_per_run.push(txs_per_run);
    }

    pub fn mean(values: &[U256]) -> U256 {
        let sum: U256 = sum_vec_u256(values);
        sum / values.len()
    }

    pub fn median(values: &[U256]) -> eyre::Result<U256> {
        let mid = values.len() / 2;
        if values.len() % 2 == 0 {
            Ok((*values.get(mid - 1).context("Indexing error")?
                + *values.get(mid).context("Indexing error")?)
                / 2_u32)
        } else {
            values.get(mid).context("Indexing error").copied()
        }
    }

    #[allow(clippy::as_conversions, reason = "Allow as conversion for usize")]
    pub fn std_deviation(values: &[U256], units: i32) -> eyre::Result<f64> {
        let float_values = values
            .iter()
            .map(|v| {
                format_units(*v, units)
                    .context("Failed to format units")
                    .and_then(|f| {
                        f.parse::<f64>()
                            .context("Failed to parse formatted value as f64")
                    })
            })
            .collect::<eyre::Result<Vec<f64>>>()?;

        // Calculate the Variance (σ²) = (1 / (n - 1)) × Σ (x_i - μ)²
        let len = float_values.len();
        // Revise this as_conversion
        let f64_len: f64 = len as f64;
        let variance: f64 = if len == 1 {
            0.0
        } else {
            // Calculate the mean (μ)
            let mean: f64 = float_values.iter().sum::<f64>() / f64_len;
            float_values
                .iter()
                .map(|value| {
                    let diff = value - mean;
                    diff * diff
                })
                .sum::<f64>()
                / (f64_len - 1.0)
        };

        // Calculate the standard deviation (σ)
        let std_dev = variance.sqrt();

        Ok(std_dev)
    }
}

impl fmt::Display for GasTracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let gas_mean = Self::mean(&self.gas);
        let gas_median = Self::median(&self.gas).unwrap_or_else(|_| U256::zero());

        let fees_mean = Self::mean(&self.fees);
        let fees_median = Self::median(&self.fees).unwrap_or_else(|_| U256::zero());
        let formatted_fees_mean =
            format_units(fees_mean, self.token_decimals).unwrap_or("0".to_owned());
        let formatted_fees_median =
            format_units(fees_median, self.token_decimals).unwrap_or("0".to_owned());
        let fees_std_dev = Self::std_deviation(&self.fees, self.token_decimals).unwrap_or(0.0_f64);

        let gas_price_mean = Self::mean(&self.gas_prices);
        let gas_price_median = Self::median(&self.gas_prices).unwrap_or_else(|_| U256::zero());
        let formatted_gas_price_mean =
            format_units(gas_price_mean, 9_i32).unwrap_or("0".to_owned());
        let formatted_gas_price_median =
            format_units(gas_price_median, 9_i32).unwrap_or("0".to_owned());
        let gas_price_std_dev = Self::std_deviation(&self.gas_prices, 9_i32).unwrap_or(0.0_f64);

        let total_txs = self.txs_per_run.iter().sum::<u64>();
        let mu = "μ".bright_red();
        let sd = "σ".bright_red();
        let md = "Md".bright_red();

        writeln!(f, "GasTracker Statistics:")?;
        writeln!(
            f,
            "{}:\n {mu}: {}, {md}: {}",
            "Gas Used".bright_cyan().on_black(),
            gas_mean,
            gas_median
        )?;
        let s = self.token_symbol.bright_yellow().on_black();
        writeln!(
            f,
            "{}:\n {mu}: {:.10} {s}, {sd}: {:.10} {s}, {md}: {:.10} {s}",
            "Fees".bright_cyan().on_black(),
            formatted_fees_mean,
            fees_std_dev,
            formatted_fees_median,
        )?;
        let s = "1e9".bright_yellow().on_black();
        writeln!(
            f,
            "{}:\n {mu}: {:.10} {s}, {sd}: {:.10} {s}, {md}: {:.10} {s}",
            "Gas Prices".bright_cyan().on_black(),
            formatted_gas_price_mean,
            gas_price_std_dev,
            formatted_gas_price_median,
        )?;
        writeln!(f, "Total Transactions: {total_txs}")?;
        writeln!(
            f,
            "Transactions per Run: {}",
            self.txs_per_run.first().unwrap_or(&0)
        )?;
        Ok(())
    }
}
