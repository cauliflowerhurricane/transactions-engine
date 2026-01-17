use std::collections::{HashMap, hash_map};

use fastnum::*;

use crate::domain::{AccountState, CustomerId, Transaction, TransactionId};

#[derive(Default)]
pub struct AccountingEngine {
    accounts: HashMap<CustomerId, Account>,
    transactions: HashMap<TransactionId, TransactionState>,
}

#[derive(Default, PartialEq, Eq, Debug)]
struct Account {
    available: D128,
    held: D128,
    locked: bool,
}

enum TransactionState {
    Deposited { client: CustomerId, amount: D128 },
    Withdrawed,
    Disputed { client: CustomerId, amount: D128 },
    Resolved,
    ChargedBack,
}

impl AccountingEngine {
    /// Handles a transaction and updates the accounting engine's state accordingly.
    ///
    /// Returns true if the transaction was processed successfully, and false otherwise.
    ///
    /// Disputes can only be raised on deposits.
    ///
    /// If an account is locked, no further deposits or withdrawals can be processed for that account,
    /// but disputes, resolves, and chargebacks can still be processed.
    pub fn handle_transaction(&mut self, tx: Transaction) -> bool {
        match tx {
            Transaction::Deposit { client, tx, amount } => self.deposit(client, tx, amount),
            Transaction::Withdrawal { client, tx, amount } => self.withdraw(client, tx, amount),
            Transaction::Dispute { client, tx } => self.dispute(client, tx),
            Transaction::Resolve { client, tx } => self.resolve(client, tx),
            Transaction::Chargeback { client, tx } => self.chargeback(client, tx),
        }
    }

    pub fn account_states(&self) -> Vec<AccountState> {
        self.accounts
            .iter()
            .map(|(&client, account)| AccountState {
                client,
                available: account.available,
                held: account.held,
                total: account.available + account.held,
                locked: account.locked,
            })
            .collect()
    }

    fn deposit(&mut self, client: CustomerId, tx: TransactionId, amount: D128) -> bool {
        let account = self.accounts.entry(client).or_default();
        if account.locked {
            return false;
        }

        match self.transactions.entry(tx) {
            hash_map::Entry::Occupied(_) => return false,
            hash_map::Entry::Vacant(entry) => {
                entry.insert(TransactionState::Deposited { client, amount })
            }
        };

        account.available += amount;
        true
    }

    fn withdraw(&mut self, client: CustomerId, tx: TransactionId, amount: D128) -> bool {
        let account = self.accounts.entry(client).or_default();
        if account.locked || account.available < amount {
            return false;
        }

        match self.transactions.entry(tx) {
            hash_map::Entry::Occupied(_) => return false,
            hash_map::Entry::Vacant(entry) => entry.insert(TransactionState::Withdrawed),
        };

        account.available -= amount;
        true
    }

    fn dispute(&mut self, client: CustomerId, tx: TransactionId) -> bool {
        let Some(transaction) = self.transactions.get_mut(&tx) else {
            return false;
        };

        let TransactionState::Deposited {
            client: deposit_client,
            amount,
        } = *transaction
        else {
            return false;
        };

        if deposit_client != client {
            return false;
        }

        let account = self
            .accounts
            .get_mut(&client)
            .expect("Account must exist since the transaction exists");

        account.held += amount;
        account.available -= amount;
        *transaction = TransactionState::Disputed { client, amount };
        true
    }

    fn resolve(&mut self, client: CustomerId, tx: TransactionId) -> bool {
        let Some(transaction) = self.transactions.get_mut(&tx) else {
            return false;
        };

        let TransactionState::Disputed {
            client: deposit_client,
            amount,
        } = *transaction
        else {
            return false;
        };

        if deposit_client != client {
            return false;
        }

        let account = self
            .accounts
            .get_mut(&client)
            .expect("Account must exist since the transaction exists");

        account.held -= amount;
        account.available += amount;
        *transaction = TransactionState::Resolved;
        true
    }

    fn chargeback(&mut self, client: CustomerId, tx: TransactionId) -> bool {
        let Some(transaction) = self.transactions.get_mut(&tx) else {
            return false;
        };

        let TransactionState::Disputed {
            client: deposit_client,
            amount,
        } = *transaction
        else {
            return false;
        };

        if deposit_client != client {
            return false;
        }

        let account = self
            .accounts
            .get_mut(&client)
            .expect("Account must exist since the transaction exists");

        account.held -= amount;
        account.locked = true;
        *transaction = TransactionState::ChargedBack;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIENT: CustomerId = CustomerId::new(1);

    #[test]
    fn test_deposit() {
        let mut engine = AccountingEngine::default();
        assert!(engine.deposit(CLIENT, TransactionId::new(1), dec128!(1.0)));
        check_account(&engine, CLIENT, dec128!(1.0), dec128!(0.0), false);
    }

    #[test]
    fn test_deposit_withdraw_all() {
        let mut engine = AccountingEngine::default();
        assert!(engine.deposit(CLIENT, TransactionId::new(1), dec128!(1.0)));
        assert!(engine.withdraw(CLIENT, TransactionId::new(2), dec128!(1.0)));
        check_account(&engine, CLIENT, dec128!(0.0), dec128!(0.0), false);
    }

    #[test]
    fn test_deposit_withdraw_insufficient_funds() {
        let mut engine = AccountingEngine::default();
        assert!(engine.deposit(CLIENT, TransactionId::new(1), dec128!(1.0)));
        assert!(!engine.withdraw(CLIENT, TransactionId::new(2), dec128!(2.0)));
        check_account(&engine, CLIENT, dec128!(1.0), dec128!(0.0), false);
    }

    #[test]
    fn test_dispute_resolve() {
        let mut engine = AccountingEngine::default();
        assert!(engine.deposit(CLIENT, TransactionId::new(1), dec128!(1.0)));

        assert!(engine.dispute(CLIENT, TransactionId::new(1)));
        check_account(&engine, CLIENT, dec128!(0.0), dec128!(1.0), false);

        assert!(engine.resolve(CLIENT, TransactionId::new(1)));
        check_account(&engine, CLIENT, dec128!(1.0), dec128!(0.0), false);
    }

    #[test]
    fn test_dispute_chargeback() {
        let mut engine = AccountingEngine::default();
        assert!(engine.deposit(CLIENT, TransactionId::new(1), dec128!(1.0)));

        assert!(engine.dispute(CLIENT, TransactionId::new(1)));
        check_account(&engine, CLIENT, dec128!(0.0), dec128!(1.0), false);

        assert!(engine.chargeback(CLIENT, TransactionId::new(1)));
        check_account(&engine, CLIENT, dec128!(0.0), dec128!(0.0), true);
    }

    #[test]
    fn test_dispute_twice() {
        let mut engine = AccountingEngine::default();
        assert!(engine.deposit(CLIENT, TransactionId::new(1), dec128!(1.0)));

        assert!(engine.dispute(CLIENT, TransactionId::new(1)));
        assert!(engine.resolve(CLIENT, TransactionId::new(1)));

        assert!(!engine.dispute(CLIENT, TransactionId::new(1)));
        check_account(&engine, CLIENT, dec128!(1.0), dec128!(0.0), false);
    }

    #[test]
    fn test_locked() {
        let mut engine = AccountingEngine {
            accounts: HashMap::from([(
                CLIENT,
                Account {
                    available: dec128!(1.0),
                    held: dec128!(0.0),
                    locked: true,
                },
            )]),
            ..Default::default()
        };
        assert!(!engine.deposit(CLIENT, TransactionId::new(1), dec128!(1.0)));
        assert!(!engine.withdraw(CLIENT, TransactionId::new(2), dec128!(1.0)));
        assert!(!engine.dispute(CLIENT, TransactionId::new(1)));
        check_account(&engine, CLIENT, dec128!(1.0), dec128!(0.0), true);
    }

    #[test]
    fn test_dispute_resolve_unavailable_funds() {
        let mut engine = AccountingEngine::default();
        assert!(engine.deposit(CLIENT, TransactionId::new(1), dec128!(1.0)));
        assert!(engine.withdraw(CLIENT, TransactionId::new(2), dec128!(1.0)));

        assert!(engine.dispute(CLIENT, TransactionId::new(1)));
        check_account(&engine, CLIENT, dec128!(-1.0), dec128!(1.0), false);

        assert!(engine.resolve(CLIENT, TransactionId::new(1)));
        check_account(&engine, CLIENT, dec128!(0.0), dec128!(0.0), false);
    }

    #[test]
    fn test_dispute_chargeback_unavailable_funds() {
        let mut engine = AccountingEngine::default();
        assert!(engine.deposit(CLIENT, TransactionId::new(1), dec128!(1.0)));
        assert!(engine.withdraw(CLIENT, TransactionId::new(2), dec128!(1.0)));

        assert!(engine.dispute(CLIENT, TransactionId::new(1)));
        check_account(&engine, CLIENT, dec128!(-1.0), dec128!(1.0), false);

        assert!(engine.chargeback(CLIENT, TransactionId::new(1)));
        check_account(&engine, CLIENT, dec128!(-1.0), dec128!(0.0), true);
    }

    fn check_account(
        engine: &AccountingEngine,
        client: CustomerId,
        available: D128,
        held: D128,
        locked: bool,
    ) {
        let account = engine.accounts.get(&client).expect("Account must exist");
        assert_eq!(
            account,
            &Account {
                available,
                held,
                locked,
            },
        );
    }
}
