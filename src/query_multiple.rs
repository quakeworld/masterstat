use futures::future;
use std::time::Duration;

use crate::query;

#[derive(Debug, Default)]
pub struct QuerySuccess {
    master_address: String,
    server_addresses: Vec<String>,
}
impl QuerySuccess {
    pub fn master_address(&self) -> &str {
        &self.master_address
    }

    pub fn server_addresses(&self) -> &[String] {
        &self.server_addresses
    }
}

#[derive(Debug)]
pub struct QueryFailure {
    master_address: String,
    error: anyhow::Error,
}
impl QueryFailure {
    pub fn master_address(&self) -> &str {
        &self.master_address
    }

    pub fn error(&self) -> &anyhow::Error {
        &self.error
    }
}

#[derive(Debug, Default)]
pub struct MultiQueryResult {
    /// Collection of successful queries.
    successes: Vec<QuerySuccess>,

    /// Collection of failed queries.
    failures: Vec<QueryFailure>,
}

impl MultiQueryResult {
    /// Iterator over successful query results.
    pub fn successful_queries(&self) -> impl Iterator<Item = &QuerySuccess> {
        self.successes.iter()
    }

    /// Iterator over failed query results.
    pub fn failed_queries(&self) -> impl Iterator<Item = &QueryFailure> {
        self.failures.iter()
    }

    /// Unique server addresses from successful queries.
    pub fn server_addresses(&self) -> Vec<String> {
        let mut addresses: Vec<String> = self
            .successes
            .iter()
            .flat_map(|res| res.server_addresses().iter())
            .cloned()
            .collect();
        addresses.sort();
        addresses.dedup();
        addresses
    }
}

/// Get server addresses from multiple master servers (concurrently)
///
/// # Arguments
/// * `master_addresses` - A slice of master server addresses to query.
/// * `timeout` - The timeout duration for each query.
///
/// # Returns
/// A `MultiQueryResult` containing successful and failed queries.
///
/// # Example
/// ```rust
/// #[tokio::main]
/// async fn main() {
///     let master_addresses = vec![
///         "master.quakeservers.net:27000".to_string(),
///         "master.quakeworld.nu:27000".to_string(),
///     ];
///     let timeout = std::time::Duration::from_secs(2);
///     let result = masterstat::query_multiple(&master_addresses, timeout).await;
/// }
/// ```
pub async fn query_multiple(master_addresses: &[String], timeout: Duration) -> MultiQueryResult {
    let tasks = master_addresses
        .iter()
        .map(|address| async move { (address.clone(), query(address, timeout).await) });

    let mut results = MultiQueryResult::default();

    for (master_address, res) in future::join_all(tasks).await {
        match res {
            Ok(server_addresses) => results.successes.push(QuerySuccess {
                master_address,
                server_addresses,
            }),
            Err(error) => results.failures.push(QueryFailure {
                master_address,
                error,
            }),
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    #[cfg_attr(feature = "ci", ignore)]
    async fn test_query_multiple() -> Result<()> {
        let master_addresses = vec![
            "master.quakeservers.net:27000".to_string(),
            "master.quakeworld.nu:27000".to_string(),
            "INVALID:27000".to_string(),
        ];
        let results = query_multiple(&master_addresses, Duration::from_secs(2)).await;

        assert!(results.server_addresses().len() >= 300);
        assert!(2 == results.successful_queries().count());
        assert!(1 == results.failed_queries().count());

        let query1 = results.successful_queries().next().unwrap();
        assert!(query1.server_addresses().len() >= 300);
        assert_eq!(
            query1.master_address(),
            "master.quakeservers.net:27000".to_string()
        );

        let query2 = results.successful_queries().last().unwrap();
        assert!(query2.server_addresses().len() >= 300);
        assert_eq!(
            query2.master_address(),
            "master.quakeworld.nu:27000".to_string()
        );

        let query3 = results.failed_queries().next().unwrap();
        assert_eq!(query3.master_address(), "INVALID:27000".to_string());
        assert!(query3.error().to_string().contains("failed to lookup"));

        Ok(())
    }
}
