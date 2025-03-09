use crate::{account::Account, error::TransactorError, manager::Manager};
use serde::{Serialize, Serializer};
use std::io::{stdout, BufWriter};

/// The representation of a CSV account record.
#[derive(Debug, Serialize)]
struct AccountRecord {
    client: u16,
    #[serde(serialize_with = "fixed_point_serialize")]
    available: u64,
    #[serde(serialize_with = "fixed_point_serialize")]
    held: u64,
    #[serde(serialize_with = "fixed_point_serialize")]
    total: u64,
    locked: bool,
}

/// Represent a u64 as a decimal with a specified precision.
/// In this case assuming decimal with precision to the ten thousandths place.
fn fixed_point_serialize<S>(x: &u64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let scale = 10_000;
    let whole = x / scale;
    let fract = x % scale;
    s.serialize_str(&format!("{whole}.{fract:04}"))
}

impl From<(u16, Account)> for AccountRecord {
    fn from((client, acct): (u16, Account)) -> Self {
        AccountRecord {
            client,
            available: acct.available(),
            held: acct.held(),
            total: acct.total(),
            locked: acct.is_frozen(),
        }
    }
}

/// For each account record in the `Manager`, serialize and write it to stdout.
pub fn unload_data(manager: Manager) -> Result<(), TransactorError> {
    let writer = BufWriter::new(stdout());
    let mut wtr = csv::Writer::from_writer(writer);

    for client in manager {
        let record: AccountRecord = client.into();
        wtr.serialize(record)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::AccountRecord;

    #[test]
    fn serilaize() {
        let buf = Vec::new();
        let mut wtr = csv::Writer::from_writer(buf);

        let record = AccountRecord {
            client: 1,
            available: 10000,
            held: 5000,
            total: 15000,
            locked: false,
        };

        wtr.serialize(record).expect("Failed to serialize");
        let inner = wtr.into_inner().unwrap();
        let str = std::str::from_utf8(&inner).expect("Failed to convert");
        assert_eq!(
            str,
            "client,available,held,total,locked\n1,1.0000,0.5000,1.5000,false\n"
        )
    }
}
