use rust_decimal::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct ClientAccount {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
    #[serde(skip_serializing)]
    pub disputes: HashSet<u32>,
    #[serde(skip_serializing)]
    pub completed_disputes: HashSet<u32>,
    #[serde(skip_serializing)]
    pub transactions: HashMap<u32, InternalTransaction>,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct InternalTransaction {
    pub kind: TransactionType,
    pub amount: Decimal,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct ReadTransaction {
    pub kind: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<Decimal>,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum TransactionType {
    Chargeback,
    Deposit,
    Dispute,
    Resolve,
    Withdrawal,
}

impl FromStr for TransactionType {
    type Err = ();

    fn from_str(input: &str) -> Result<TransactionType, Self::Err> {
        match input.to_lowercase().as_str() {
            "chargeback" => Ok(TransactionType::Chargeback),
            "deposit" => Ok(TransactionType::Deposit),
            "dispute" => Ok(TransactionType::Dispute),
            "resolve" => Ok(TransactionType::Resolve),
            "withdrawal" => Ok(TransactionType::Withdrawal),
            _ => Err(()),
        }
    }
}
