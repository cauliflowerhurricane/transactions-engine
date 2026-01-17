use color_eyre::eyre::{self, OptionExt as _, ensure};

use crate::domain::{Transaction, TransactionRow, TransactionType};

impl TryFrom<TransactionRow> for Transaction {
    type Error = eyre::Report;

    fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
        match row.r#type {
            TransactionType::Deposit => {
                let amount = row
                    .amount
                    .ok_or_eyre("Amount is required for 'deposit' transactions")?;
                Ok(Transaction::Deposit {
                    client: row.client,
                    tx: row.tx,
                    amount,
                })
            }
            TransactionType::Withdrawal => {
                let amount = row
                    .amount
                    .ok_or_eyre("Amount is required for 'withdrawal' transactions")?;
                Ok(Transaction::Withdrawal {
                    client: row.client,
                    tx: row.tx,
                    amount,
                })
            }
            TransactionType::Dispute => {
                ensure!(
                    row.amount.is_none(),
                    "Amount must not be provided for 'dispute' transactions"
                );
                Ok(Transaction::Dispute {
                    client: row.client,
                    tx: row.tx,
                })
            }
            TransactionType::Resolve => {
                ensure!(
                    row.amount.is_none(),
                    "Amount must not be provided for 'resolve' transactions"
                );
                Ok(Transaction::Resolve {
                    client: row.client,
                    tx: row.tx,
                })
            }
            TransactionType::Chargeback => {
                ensure!(
                    row.amount.is_none(),
                    "Amount must not be provided for 'chargeback' transactions"
                );
                Ok(Transaction::Chargeback {
                    client: row.client,
                    tx: row.tx,
                })
            }
        }
    }
}
