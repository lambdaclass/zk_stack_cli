use eyre::ContextCompat;
use std::fmt;
use zksync_ethers_rs::types::U256;

#[derive(Debug, Clone)]
pub struct GasTracker {
    gas: Vec<U256>,
    fees: Vec<U256>,
    gas_prices: Vec<U256>,
    txs_per_run: Vec<u64>,
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
        }
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

    #[allow(dead_code)]
    pub fn print_parsed_statistics(&self) -> eyre::Result<()> {
        let gas_mean = Self::mean(&self.gas);
        let gas_median = Self::median(&self.gas)?;

        let fees_mean = Self::mean(&self.fees);
        let fees_median = Self::median(&self.fees)?;

        let gas_price_mean = Self::mean(&self.gas_prices);
        let gas_price_median = Self::median(&self.gas_prices)?;

        println!("Gas Used: Mean: {gas_mean:.2}, Median: {gas_median:.2}");
        println!("Fees: Mean: {fees_mean:.2}, Median: {fees_median:.2}");
        println!("Gas Prices: Mean: {gas_price_mean:.2}, Median: {gas_price_median:.2}");
        Ok(())
    }
}

impl fmt::Display for GasTracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let gas_mean = Self::mean(&self.gas);
        let gas_median = Self::median(&self.gas).unwrap_or_else(|_| U256::zero());

        let fees_mean = Self::mean(&self.fees);
        let fees_median = Self::median(&self.fees).unwrap_or_else(|_| U256::zero());

        let gas_price_mean = Self::mean(&self.gas_prices);
        let gas_price_median = Self::median(&self.gas_prices).unwrap_or_else(|_| U256::zero());

        let total_txs = self.txs_per_run.iter().sum::<u64>();
        writeln!(f, "GasTracker Statistics:")?;
        writeln!(f, "Gas Used: Mean: {gas_mean}, Median: {gas_median}")?;
        writeln!(f, "Fees: Mean: {fees_mean}, Median: {fees_median}")?;
        writeln!(
            f,
            "Gas Prices: Mean: {gas_price_mean}, Median: {gas_price_median}"
        )?;
        writeln!(f, "Total Transactions: {total_txs}")?;
        writeln!(f, "Transactions per Run: {:?}\n", self.txs_per_run)?;

        Ok(())
    }
}
