use crate::{account::Account, error::TransactorError};
use std::collections::{
    hash_map::{IntoIter, Iter},
    HashMap,
};

/// Account manager associating a client ID to an account.
#[derive(Debug)]
pub struct Manager {
    accounts: HashMap<u16, Account>,
}

impl Default for Manager {
    fn default() -> Self {
        Manager::new()
    }
}

impl Manager {
    /// Construct a new `Manager`.
    pub fn new() -> Self {
        Manager {
            accounts: HashMap::new(),
        }
    }

    /// Deposit funds into the account specified by the client ID.
    pub fn deposit(&mut self, client: u16, tx: u32, amt: u64) -> Result<(), TransactorError> {
        if let Some(acct) = self.accounts.get_mut(&client) {
            acct.deposit(tx, amt)?;
            return Ok(());
        }

        let acct = Account::new(tx, amt);
        self.accounts.insert(client, acct);

        Ok(())
    }

    /// Withdraw funds from the account specified by the client ID.
    pub fn withdraw(&mut self, client: u16, amt: u64) -> Result<(), TransactorError> {
        let account = self
            .accounts
            .get_mut(&client)
            .ok_or(TransactorError::NoClient(client))?;
        account.withdraw(amt)?;

        Ok(())
    }

    /// Dispute a transaction according to the client and transaction ID pair
    pub fn dispute(&mut self, client: u16, tx: u32) -> Result<(), TransactorError> {
        let account = self
            .accounts
            .get_mut(&client)
            .ok_or(TransactorError::NoClient(client))?;

        account.dispute(tx)?;

        Ok(())
    }

    /// Resolve a dispute according to the client and transaction ID pair
    pub fn resolve(&mut self, client: u16, tx: u32) -> Result<(), TransactorError> {
        let account = self
            .accounts
            .get_mut(&client)
            .ok_or(TransactorError::NoClient(client))?;
        account.resolve(tx)?;

        Ok(())
    }

    /// Chargeback a disputed transaction according to the client and transaction ID pair
    pub fn chargeback(&mut self, client: u16, tx: u32) -> Result<(), TransactorError> {
        let account = self
            .accounts
            .get_mut(&client)
            .ok_or(TransactorError::NoClient(client))?;
        account.chargeback(tx)?;

        Ok(())
    }
}

impl IntoIterator for Manager {
    type Item = (u16, Account);
    type IntoIter = IntoIter<u16, Account>;

    fn into_iter(self) -> Self::IntoIter {
        self.accounts.into_iter()
    }
}

impl<'a> IntoIterator for &'a Manager {
    type Item = (&'a u16, &'a Account);
    type IntoIter = Iter<'a, u16, Account>;

    fn into_iter(self) -> Self::IntoIter {
        self.accounts.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::Manager;
    use crate::error::TransactorError;

    fn validate_accounts(mgr: &Manager, clients: &[u16]) {
        assert_eq!(mgr.accounts.len(), clients.len());

        for client in clients {
            assert!(mgr.accounts.contains_key(client));
        }
    }

    #[test]
    fn registration() {
        let mut mgr = Manager::new();
        mgr.deposit(1, 1, 100).expect("Failed to deposit");
        mgr.deposit(2, 2, 200).expect("Failed to deposit");
        mgr.deposit(3, 3, 300).expect("Failed to deposit");
        mgr.deposit(4, 4, 400).expect("Failed to deposit");
        validate_accounts(&mgr, &[1, 2, 3, 4]);
    }

    #[test]
    fn deposit_withdrawal() {
        let mut mgr = Manager::new();
        mgr.deposit(1, 1, 100).expect("Failed to deposit");
        mgr.deposit(2, 2, 200).expect("Failed to deposit");
        mgr.withdraw(1, 50).expect("Failed to withdrawal");
        mgr.withdraw(2, 100).expect("Failed to withdrawal");
        mgr.deposit(1, 5, 100).expect("Failed to deposit");
        validate_accounts(&mgr, &[1, 2]);

        assert_eq!(mgr.accounts[&1].available(), 150);
        assert_eq!(mgr.accounts[&1].held(), 0);
        assert_eq!(mgr.accounts[&2].available(), 100);
        assert_eq!(mgr.accounts[&2].held(), 0);
    }

    #[test]
    fn dispute_resolve() {
        let mut mgr = Manager::new();
        mgr.deposit(1, 1, 100).expect("Failed to deposit");
        mgr.deposit(1, 3, 100).expect("Failed to deposit");
        mgr.deposit(2, 2, 200).expect("Failed to deposit");
        mgr.dispute(1, 3).expect("Failed to dispute");

        validate_accounts(&mgr, &[1, 2]);

        // Validate client 1
        assert_eq!(mgr.accounts[&1].available(), 100);
        assert_eq!(mgr.accounts[&1].held(), 100);
        assert_eq!(mgr.accounts[&1].total(), 200);

        // Validate client 2
        assert_eq!(mgr.accounts[&2].available(), 200);
        assert_eq!(mgr.accounts[&2].held(), 0);
        assert_eq!(mgr.accounts[&2].total(), 200);

        // Resolve transaction 3
        mgr.resolve(1, 3).expect("Failed to resolve");

        // Validate client 1 and transaction 3
        assert_eq!(mgr.accounts[&1].available(), 200);
        assert_eq!(mgr.accounts[&1].held(), 0);
        assert_eq!(mgr.accounts[&1].total(), 200);
    }

    #[test]
    fn dispute_chargeback() {
        let mut mgr = Manager::new();

        mgr.deposit(1, 1, 100).expect("Failed to deposit");
        mgr.deposit(1, 3, 100).expect("Failed to deposit");
        mgr.deposit(2, 2, 200).expect("Failed to deposit");
        mgr.dispute(1, 3).expect("Failed to dispute");

        validate_accounts(&mgr, &[1, 2]);

        // Validate cient 1
        assert_eq!(mgr.accounts[&1].available(), 100);
        assert_eq!(mgr.accounts[&1].held(), 100);
        assert_eq!(mgr.accounts[&1].total(), 200);
        assert!(!mgr.accounts[&1].is_frozen());

        // Validate client 2
        assert_eq!(mgr.accounts[&2].available(), 200);
        assert_eq!(mgr.accounts[&2].held(), 0);
        assert_eq!(mgr.accounts[&2].total(), 200);

        // Chargeback transaction 3
        mgr.chargeback(1, 3).expect("Failed to chargeback");

        // Validate client 1 and transaction 3
        assert_eq!(mgr.accounts[&1].available(), 100);
        assert_eq!(mgr.accounts[&1].held(), 0);
        assert_eq!(mgr.accounts[&1].total(), 100);
        assert!(mgr.accounts[&1].is_frozen());

        // Atttempt to interact with client 1
        let result = mgr.withdraw(1, 50);
        assert!(matches!(result, Err(TransactorError::FrozenAccount)))
    }
}
