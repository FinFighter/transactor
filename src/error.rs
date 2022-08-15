use std::{error, fmt, io};

#[derive(Debug)]
pub enum TransactorError {
    /// A failure during parsing.
    ParseError(csv::Error),

    /// IO error occured while reading from a file or writing
    /// results to stdout.
    IoError(io::Error),

    /// A deposit or withdrawal transaction omitted the amount
    MissingAmount,

    /// A withdrawal exceeds the available funds in the account.
    WithdrawalExceedsAvailable { available: u64, attempted: u64 },

    /// A dispute exceeds the available funds in the account.
    DisputeExceedsAvailable { available: u64, attempted: u64 },

    /// The account is frozen, no further actions may effect it.
    FrozenAccount,

    /// The client ID does not match an active account.
    NoClient(u16),

    /// The transaction ID does not match a previous transaction.
    NoTransaction(u32),

    /// A deposit or withdrawal transaction duplicated a transaction ID
    DuplicateTxn(u32),

    /// A resolve or chargeback action attempted on an non disputed transaction.
    NonDisputedTxn(u32),

    /// Attempt to dispute an already disputed transaction.
    AlreadyDisputedTxn(u32),
}

impl TransactorError {
    /// Construct a WithdrawalExceedsAvailable error.
    pub fn withdrawal_exceeds(available: u64, attempted: u64) -> Self {
        TransactorError::WithdrawalExceedsAvailable {
            available,
            attempted,
        }
    }

    /// Construct a DisputeExceedsAvailable error.
    pub fn dispute_exceeds(available: u64, attempted: u64) -> Self {
        TransactorError::DisputeExceedsAvailable {
            available,
            attempted,
        }
    }
}

impl fmt::Display for TransactorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactorError::IoError(err) => write!(f, "io error: {err}"),
            TransactorError::ParseError(err) => write!(f, "parse error: {err}"),
            TransactorError::MissingAmount => write!(
                f,
                "missing an amount with a deposit or withdrawal operation"
            ),
            TransactorError::WithdrawalExceedsAvailable {
                available,
                attempted,
            } => write!(
                f,
                "attempt to debit amount of {attempted} exceeds avaiable funds of {available}"
            ),
            TransactorError::DisputeExceedsAvailable {
                available,
                attempted,
            } => write!(
                f,
                "attempt to dispute amount of {attempted} exceeds avaiable funds of {available}"
            ),
            TransactorError::FrozenAccount => write!(f, "account is frozen"),
            TransactorError::NoClient(id) => {
                write!(f, "client with id {id} does not exist")
            }
            TransactorError::NoTransaction(id) => {
                write!(f, "transaction with id {id} does not exist")
            }
            TransactorError::DuplicateTxn(id) => {
                write!(f, "transaction with id {id} already exists")
            }
            TransactorError::NonDisputedTxn(id) => {
                write!(f, "transaction with id {id} is not disputed")
            }
            TransactorError::AlreadyDisputedTxn(id) => {
                write!(f, "transaction with id {id} is already disputed")
            }
        }
    }
}

impl From<io::Error> for TransactorError {
    fn from(error: io::Error) -> Self {
        TransactorError::IoError(error)
    }
}

impl From<csv::Error> for TransactorError {
    fn from(error: csv::Error) -> Self {
        TransactorError::ParseError(error)
    }
}

impl error::Error for TransactorError {}
