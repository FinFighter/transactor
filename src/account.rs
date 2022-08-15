use std::collections::{hash_map::Entry, HashMap};

use crate::error::TransactorError;

/// A deposit transaction tracking the amount and whether its disputed.
#[derive(Debug)]
struct Deposit {
    amount: u64,
    disputed: bool,
}

impl Deposit {
    /// Construct a new `Deposit` transaction.
    fn new(amount: u64) -> Self {
        Deposit {
            amount,
            disputed: false,
        }
    }

    /// Return whether the deposit is disputed.
    #[inline]
    fn is_disputed(&self) -> bool {
        self.disputed
    }

    /// Get the amount of funds this deposit represents.
    #[inline]
    fn amount(&self) -> u64 {
        self.amount
    }

    /// Set the `Deposit` transaction to disputed.
    #[inline]
    fn dispute(&mut self) {
        self.disputed = true;
    }

    /// Set the `Deposit` transaction to resolved.
    #[inline]
    fn resolve(&mut self) {
        self.disputed = false;
    }
}

/// A client account that maintains the historical deposits and current funds.
#[derive(Debug)]
pub struct Account {
    available: u64,
    held: u64,
    frozen: bool,
    deposits: HashMap<u32, Deposit>,
}

impl Account {
    /// Create a new `Account` with an initial deposit.
    #[inline]
    pub fn new(tx: u32, available: u64) -> Self {
        let mut deposits = HashMap::new();
        deposits.insert(tx, Deposit::new(available));

        Account {
            available,
            held: 0,
            frozen: false,
            deposits,
        }
    }

    /// Get the available funds.
    #[inline]
    pub fn available(&self) -> u64 {
        self.available
    }

    /// Get the held funds.
    #[inline]
    pub fn held(&self) -> u64 {
        self.held
    }

    /// Get the total funds.
    #[inline]
    pub fn total(&self) -> u64 {
        self.available + self.held
    }

    /// Return whether the account is frozen.
    #[inline]
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// Deposit funds into the `Account`.
    /// If the account is frozen or there is a duplicate transaction id, the action will not execute.
    #[inline]
    pub fn deposit(&mut self, tx: u32, amt: u64) -> Result<(), TransactorError> {
        if self.frozen {
            return Err(TransactorError::FrozenAccount);
        }

        if let Entry::Vacant(entry) = self.deposits.entry(tx) {
            entry.insert(Deposit::new(amt));
            self.available += amt;
            return Ok(());
        }

        Err(TransactorError::DuplicateTxn(tx))
    }

    /// Withdraw funds from the `Account`.
    /// If the account is frozen or there is a lack of funds, the action will not execute.
    #[inline]
    pub fn withdraw(&mut self, amt: u64) -> Result<(), TransactorError> {
        if self.frozen {
            return Err(TransactorError::FrozenAccount);
        }

        if self.available < amt {
            return Err(TransactorError::withdrawal_exceeds(self.available, amt));
        }

        self.available -= amt;
        Ok(())
    }

    /// Dispute a previously processed deposit.
    /// If the account is frozen or there is a duplicate transaction id, the action will not execute.
    #[inline]
    pub fn dispute(&mut self, tx: u32) -> Result<(), TransactorError> {
        if self.frozen {
            return Err(TransactorError::FrozenAccount);
        }

        let deposit = self
            .deposits
            .get_mut(&tx)
            .ok_or(TransactorError::NoTransaction(tx))?;
        let amt = deposit.amount();

        if deposit.is_disputed() {
            return Err(TransactorError::AlreadyDisputedTxn(tx));
        }

        if self.available < amt {
            return Err(TransactorError::dispute_exceeds(self.available, amt));
        }

        deposit.dispute();

        self.available -= amt;
        self.held += amt;
        Ok(())
    }

    /// Resolve a disputed deposit transaction, transfering funds from held to available.
    /// If the account is frozen, there is a duplicate transaction id,
    /// or the transaction is not disputed, the action will not execute.
    #[inline]
    pub fn resolve(&mut self, tx: u32) -> Result<(), TransactorError> {
        if self.frozen {
            return Err(TransactorError::FrozenAccount);
        }

        let deposit = self
            .deposits
            .get_mut(&tx)
            .ok_or(TransactorError::NoTransaction(tx))?;

        if !deposit.disputed {
            return Err(TransactorError::NonDisputedTxn(tx));
        }

        let amt = deposit.amount();
        deposit.resolve();

        self.held -= amt;
        self.available += amt;
        Ok(())
    }

    /// Chargeback a disputed transaction removing the funds from the account total and locking the account.
    /// If the account is frozen, there is a duplicate transaction id,
    /// or the transaction is not disputed, the action will not execute.
    #[inline]
    pub fn chargeback(&mut self, tx: u32) -> Result<(), TransactorError> {
        if self.frozen {
            return Err(TransactorError::FrozenAccount);
        }

        let deposit = self
            .deposits
            .get_mut(&tx)
            .ok_or(TransactorError::NoTransaction(tx))?;

        if !deposit.is_disputed() {
            return Err(TransactorError::NonDisputedTxn(tx));
        }

        let amt = deposit.amount();
        deposit.resolve();

        self.held -= amt;
        self.frozen = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::TransactorError;

    use super::Account;

    fn check_account(acct: &Account, avail: u64, held: u64, frozen: bool) {
        assert_eq!(acct.available, avail);
        assert_eq!(acct.held, held);
        assert_eq!(acct.frozen, frozen);
        assert_eq!(acct.total(), avail + held);
    }

    fn check_deposit(acct: &Account, tx: u32, disputed: bool) {
        assert_eq!(acct.deposits[&tx].disputed, disputed)
    }

    #[test]
    fn deposit() {
        let mut acct = Account::new(1, 0);
        let result = acct.deposit(1, 100);

        assert!(matches!(result, Err(TransactorError::DuplicateTxn(1))));

        check_account(&acct, 0, 0, false);

        acct.deposit(2, 100).expect("Failed to deposit");

        check_account(&acct, 100, 0, false);
    }

    #[test]
    fn withdraw() {
        let mut acct = Account::new(1, 100);
        acct.withdraw(99).expect("Failed to withdraw");

        check_account(&acct, 1, 0, false)
    }

    #[test]
    fn dispute_resolve() {
        let mut acct = Account::new(1, 100);
        acct.dispute(1).unwrap();

        check_account(&acct, 0, 100, false);
        check_deposit(&acct, 1, true);

        acct.resolve(1).unwrap();

        check_account(&acct, 100, 0, false);
    }

    #[test]
    fn chargeback() {
        let mut acct = Account::new(1, 100);
        acct.dispute(1).unwrap();
        acct.chargeback(1).unwrap();

        check_account(&acct, 0, 0, true);
        check_deposit(&acct, 1, false);
    }

    #[test]
    fn double_dispute() {
        let mut acct = Account::new(1, 100);
        acct.dispute(1).unwrap();
        let result = acct.dispute(1);

        assert!(matches!(
            result,
            Err(TransactorError::AlreadyDisputedTxn(1))
        ));
        check_account(&acct, 0, 100, false);
        check_deposit(&acct, 1, true);
    }

    #[test]
    fn locked_account() {
        let mut acct = Account::new(1, 100);
        acct.dispute(1).unwrap();
        acct.chargeback(1).unwrap();
        let result = acct.deposit(2, 50);

        assert!(matches!(result, Err(TransactorError::FrozenAccount)));
        check_account(&acct, 0, 0, true);
        check_deposit(&acct, 1, false);
    }
}
