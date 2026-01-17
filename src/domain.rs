mod impls;

use fastnum::D128;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CustomerId(u16);

impl CustomerId {
    #[allow(unused)]
    pub const fn new(id: u16) -> Self {
        CustomerId(id)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TransactionId(u32);

impl TransactionId {
    #[allow(unused)]
    pub const fn new(id: u32) -> Self {
        TransactionId(id)
    }
}

#[derive(Deserialize, Debug)]
// I would expect #[serde(tag = "type")] to work here,
// but for some reason it doesn't play well with the CSV deserializer,
// so I used a helper struct and implemented `TryFrom` instead.
#[serde(try_from = "TransactionRow")]
pub enum Transaction {
    Deposit {
        client: CustomerId,
        tx: TransactionId,
        amount: D128,
    },
    Withdrawal {
        client: CustomerId,
        tx: TransactionId,
        amount: D128,
    },
    Dispute {
        client: CustomerId,
        tx: TransactionId,
    },
    Resolve {
        client: CustomerId,
        tx: TransactionId,
    },
    Chargeback {
        client: CustomerId,
        tx: TransactionId,
    },
}

#[derive(Serialize, Debug)]
pub struct AccountState {
    pub client: CustomerId,
    pub available: D128,
    pub held: D128,
    pub total: D128,
    pub locked: bool,
}

/// A helper struct to facilitate CSV deserialization of transactions.
#[derive(Deserialize, Debug)]
struct TransactionRow {
    pub r#type: TransactionType,
    pub client: CustomerId,
    pub tx: TransactionId,
    pub amount: Option<D128>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}
