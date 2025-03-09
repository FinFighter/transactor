use crate::{error::TransactorError, manager::Manager};
use serde::{Deserialize, Deserializer};
use std::{fs::File, io::BufReader};

/// The set of valid account operations.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Operation {
    Withdrawal,
    Deposit,
    Dispute,
    Resolve,
    Chargeback,
}

/// The representation of a CSV transaction record.
#[derive(Debug, Deserialize)]
struct TransactionRecord {
    #[serde(rename = "type")]
    operation: Operation,
    client: u16,
    tx: u32,
    #[serde(deserialize_with = "quantity_from_str")]
    amount: Option<u64>,
}

impl TransactionRecord {
    /// Consumes the `TransactionRecord` and applies it to the `Manager`.
    fn process(self, manager: &mut Manager) -> Result<(), TransactorError> {
        // Ignore errors resulting from manager interaction.
        // These errors are soft errors, the effects are ignored.
        // Upon encountering an error, the parsing process is allowed to continue.
        let _ = match self.operation {
            Operation::Withdrawal => {
                let amt = self.amount.ok_or(TransactorError::MissingAmount)?;
                manager.withdraw(self.client, amt)
            }
            Operation::Deposit => {
                let amt = self.amount.ok_or(TransactorError::MissingAmount)?;
                manager.deposit(self.client, self.tx, amt)
            }
            Operation::Dispute => manager.dispute(self.client, self.tx),
            Operation::Resolve => manager.resolve(self.client, self.tx),
            Operation::Chargeback => manager.chargeback(self.client, self.tx),
        };

        Ok(())
    }
}

/// Deserialize a string that resembles a floating point number into
/// a u64 scaled to the ten thousandths place.
fn quantity_from_str<'de, D>(d: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<f64> = Deserialize::deserialize(d)?;

    if let Some(amount) = value {
        if amount < 0.0 {
            return Err(serde::de::Error::custom("negative amount"));
        }

        let scale = 10_000.0;
        return Ok(Some((amount * scale).trunc() as u64));
    }

    Ok(None)
}

/// Load and deserialize data from the specified file path.
pub fn load_data(file: &str, manager: &mut Manager) -> Result<(), TransactorError> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);
    let mut rdr = csv::Reader::from_reader(reader);

    for result in rdr.deserialize() {
        let record: TransactionRecord = result?;
        record.process(manager)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Operation, TransactionRecord};
    use crate::{error::TransactorError, manager::Manager};

    const HEADER: &str = "type,client,tx,amount";

    #[test]
    fn deserialize_deposit() {
        let entry = "deposit,1,1,100";
        let csv = format!("{HEADER}\n{entry}");
        let mut rdr = csv::Reader::from_reader(csv.as_bytes());
        let mut iter = rdr.deserialize::<TransactionRecord>();
        let record = iter.next().expect("No Items").expect("Deserialize Failure");

        assert!(matches!(record.operation, Operation::Deposit));
        assert!(matches!(record.amount, Some(1000000)));
        assert_eq!(record.client, 1)
    }

    #[test]
    fn deserialize_withdraw() {
        let entry = "withdrawal,1,1,100";
        let csv = format!("{HEADER}\n{entry}");
        let mut rdr = csv::Reader::from_reader(csv.as_bytes());
        let mut iter = rdr.deserialize::<TransactionRecord>();
        let record = iter.next().expect("No Items").expect("Deserialize Failure");

        assert!(matches!(record.operation, Operation::Withdrawal));
        assert!(matches!(record.amount, Some(1000000)));
        assert_eq!(record.client, 1)
    }

    #[test]
    fn deserialize_dispute() {
        let entry = "dispute,1,1,";
        let csv = format!("{HEADER}\n{entry}");
        let mut rdr = csv::Reader::from_reader(csv.as_bytes());
        let mut iter = rdr.deserialize::<TransactionRecord>();
        let record = iter.next().expect("No Items").expect("Deserialize Failure");

        assert!(matches!(record.operation, Operation::Dispute));
        assert!(record.amount.is_none());
        assert_eq!(record.client, 1)
    }

    #[test]
    fn deserialize_resolve() {
        let entry = "resolve,1,1,";
        let csv = format!("{HEADER}\n{entry}");
        let mut rdr = csv::Reader::from_reader(csv.as_bytes());
        let mut iter = rdr.deserialize::<TransactionRecord>();
        let record = iter.next().expect("No Items").expect("Deserialize Failure");

        assert!(matches!(record.operation, Operation::Resolve));
        assert!(matches!(record.amount, None));
        assert_eq!(record.client, 1)
    }

    #[test]
    fn deserialize_chargeback() {
        let entry = "chargeback,1,1,";
        let csv = format!("{HEADER}\n{entry}");
        let mut rdr = csv::Reader::from_reader(csv.as_bytes());
        let mut iter = rdr.deserialize::<TransactionRecord>();
        let record = iter.next().expect("No Items").expect("Deserialize Failure");

        assert!(matches!(record.operation, Operation::Chargeback));
        assert!(record.amount.is_none());
        assert_eq!(record.client, 1)
    }

    #[test]
    fn truncate_long_dec() {
        let entry = "deposit,1,1,100.1234567";
        let csv = format!("{HEADER}\n{entry}");
        let mut rdr = csv::Reader::from_reader(csv.as_bytes());
        let mut iter = rdr.deserialize::<TransactionRecord>();
        let record = iter.next().expect("No Items").expect("Deserialize Failure");

        assert!(matches!(record.operation, Operation::Deposit));
        assert!(matches!(record.amount, Some(1001234)));
        assert_eq!(record.client, 1)
    }

    #[test]
    fn negative_amount() {
        let entry = "deposit,1,1,-100";
        let csv = format!("{HEADER}\n{entry}");
        let mut rdr = csv::Reader::from_reader(csv.as_bytes());
        let mut iter = rdr.deserialize::<TransactionRecord>();
        let result = iter.next().expect("No Items");
        assert!(result.is_err())
    }

    #[test]
    fn process_missing_amount() {
        let entry = "deposit,1,1,";
        let csv = format!("{HEADER}\n{entry}");
        let mut rdr = csv::Reader::from_reader(csv.as_bytes());
        let mut iter = rdr.deserialize::<TransactionRecord>();
        let record = iter.next().expect("No Items").expect("Deserialize Failure");

        let mut mgr = Manager::new();
        let result = record.process(&mut mgr);

        assert!(matches!(result, Err(TransactorError::MissingAmount)));
    }
}
