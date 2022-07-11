extern crate csv;
extern crate serde;
#[macro_use]
extern crate log;

use csv::StringRecord;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::io;
use std::process;

use rust_decimal::prelude::*;

mod model;
mod tests;
use model::*;

fn main() {
    env_logger::init();

    let input_filename = get_first_arg().unwrap().into_string().unwrap();

    // While this program is not multithreaded it would be trivial to
    // spin up a thread and execute `do_run` on its own thread.
    if let Err(err) = do_run(&input_filename, &mut io::stdout()) {
        println!("{}", err);
        process::exit(1);
    }
}

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

fn process_chargeback(account: &mut ClientAccount, transaction: ReadTransaction) {
    match account.disputes.get(&transaction.tx) {
        Some(disputed_transaction_id) => {
            match account.transactions.get_mut(disputed_transaction_id) {
                Some(disputed_transaction) => {
                    account.held -= disputed_transaction.amount;
                    account.locked = true;
                    account.disputes.remove(&transaction.tx);
                    account.completed_disputes.insert(transaction.tx);
                }
                None => {
                    error!(
                        "Unable to find disputed transaction. Referenced Transaction ID: {}",
                        disputed_transaction_id
                    );
                }
            }
        }
        None => {
            info!(
                "Attempted to chargeback transaction not in dispute. Referenced Transaction ID: {}",
                &transaction.tx
            );
        }
    }
}

fn process_deposit(account: &mut ClientAccount, transaction: ReadTransaction) {
    account.available += transaction.amount.unwrap();
    account.transactions.insert(
        transaction.tx,
        InternalTransaction {
            amount: transaction.amount.unwrap(),
            kind: transaction.kind,
        },
    );
}

fn process_dispute(account: &mut ClientAccount, transaction: ReadTransaction) {
    match account.transactions.get(&transaction.tx) {
        Some(reference_transaction) => {
            if account.disputes.contains(&transaction.tx) {
                warn!("Rejecting dispute. Referenced transaction already in dispute. Referenced Transaction ID: {}", &transaction.tx);
                return;
            }
            if account.completed_disputes.contains(&transaction.tx) {
                warn!("Rejecting dispute. Cannot dispute a transaction more than once. Referenced Transaction ID: {}", &transaction.tx);
                return;
            }

            account.held += reference_transaction.amount;
            account.available -= reference_transaction.amount;
            account.disputes.insert(transaction.tx);
        }
        None => {
            info!("Rejecting dispute. Referenced transaction not found. Referenced Transaction ID: {}", &transaction.tx);
        }
    }
}

fn process_resolve(account: &mut ClientAccount, transaction: ReadTransaction) {
    let reference_transaction_disputed = account.disputes.contains(&transaction.tx);
    if reference_transaction_disputed {
        match account.transactions.get_mut(&transaction.tx) {
            Some(reference_transaction) => {
                account.held -= reference_transaction.amount;
                account.available += reference_transaction.amount;
                account.disputes.remove(&transaction.tx);
                account.completed_disputes.insert(transaction.tx);
            }
            None => {
                error!("Rejecting resolve. Referenced transaction not found. Referenced Transaction ID: {}", &transaction.tx);
            }
        }
    } else {
        info!(
            "Rejecting resolve. Disputed transaction not found. Referenced Transaction ID: {}",
            &transaction.tx
        );
    }
}

fn process_withdrawal(account: &mut ClientAccount, transaction: ReadTransaction) {
    // Assumption - cannot dispute withdrawals that do not happen. This means
    // failed withdrawals are not saved in the transaction log.
    let transaction_amount = transaction.amount.unwrap();

    if transaction_amount <= account.available {
        account.available -= transaction_amount;
        account.transactions.insert(
            transaction.tx,
            InternalTransaction {
                amount: transaction_amount,
                kind: transaction.kind,
            },
        );
    } else {
        info!(
            "Rejecting withdrawal. Cannot withdraw more than available amount. Transaction ID: {}",
            &transaction.tx
        );
    }
}

fn process_transaction(
    client_accounts: &mut HashMap<u16, ClientAccount>,
    transaction: ReadTransaction,
) {
    let client_id = transaction.client;

    let account: &mut ClientAccount = client_accounts.entry(client_id).or_insert(ClientAccount {
        available: Decimal::new(0, 4),
        client: client_id,
        completed_disputes: HashSet::new(),
        disputes: HashSet::new(),
        held: Decimal::new(0, 4),
        locked: false,
        total: Decimal::new(0, 4),
        transactions: HashMap::new(),
    });

    // Assumption - once the account is locked we're 100% locked for this
    // client. No further transactions are processed.
    //
    // In a real life situation we would probably have to still process
    // disputes, chargebacks, and resolves.
    if account.locked {
        info!(
            "Rejecting transaction. Account locked. Referenced Transaction ID: {}",
            &transaction.tx
        );
        return;
    }

    if (transaction.kind == TransactionType::Withdrawal
        || transaction.kind == TransactionType::Deposit)
        && account.transactions.contains_key(&transaction.tx)
    {
        info!(
            "Rejecting transaction. Duplicate transaction. Transaction ID: {}",
            &transaction.tx
        );
        return;
    }

    match transaction.kind {
        TransactionType::Chargeback => process_chargeback(account, transaction),
        TransactionType::Deposit => process_deposit(account, transaction),
        TransactionType::Dispute => process_dispute(account, transaction),
        TransactionType::Resolve => process_resolve(account, transaction),
        TransactionType::Withdrawal => process_withdrawal(account, transaction),
    }
}

// Handling the record manually allows for robust CSV handling
// Serde automatic deserialization didn't like Option<Decimal> (or I couldn't
// get it work anyway)
// Serde tuple deserialization was a little better, but if the line ending in
// the CSV didn't have a comma it would throw.
//
// If this were a production system I'd add position information when logging these errors
// https://docs.rs/csv/latest/csv/struct.Reader.html#method.position
fn deserialize_transaction(record: StringRecord) -> Option<ReadTransaction> {
    let kind = if !record.is_empty() {
        match TransactionType::from_str(&record[0]) {
            Ok(val) => val,
            Err(err) => {
                error!(
                    "Rejecting transaction. Unable to read transaction type from CSV. Error: {:?}",
                    err
                );
                return None;
            }
        }
    } else {
        error!(
            "Rejecting transaction. Unable to read transaction type from CSV. Not enough fields.",
        );
        return None;
    };
    let client = if record.len() > 1 {
        match record[1].parse::<u16>() {
            Ok(val) => val,
            Err(err) => {
                error!(
                    "Rejecting transaction. Unable to read client from CSV. Error: {:?}",
                    err
                );
                return None;
            }
        }
    } else {
        error!("Rejecting transaction. Unable to read client from CSV. Not enough fields.",);
        return None;
    };
    let tx = if record.len() > 2 {
        match record[2].parse::<u32>() {
            Ok(val) => val,
            Err(err) => {
                error!(
                    "Rejecting transaction. Unable to read tx from CSV. Error: {:?}",
                    err
                );
                return None;
            }
        }
    } else {
        error!("Rejecting transaction. Unable to read tx from CSV. Not enough fields.",);
        return None;
    };
    let amount = if record.len() > 3 {
        match Decimal::from_str(&record[3]) {
            Ok(val) => Some(val),
            Err(err) => {
                if kind == TransactionType::Deposit || kind == TransactionType::Withdrawal {
                    error!(
                        "Rejecting transaction. Unable to read amount from CSV. Error: {:?}",
                        err
                    );
                    return None;
                } else {
                    None
                }
            }
        }
    } else {
        None
    };

    Some(ReadTransaction {
        kind,
        client,
        tx,
        amount,
    })
}

fn do_run(input_filename: &String, stdout: &mut dyn io::Write) -> Result<(), Box<dyn Error>> {
    let mut client_accounts: HashMap<u16, ClientAccount> = HashMap::new();

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(input_filename)?;

    for result in reader.records() {
        match result {
            Ok(r) => {
                match deserialize_transaction(r) {
                    Some(transaction) => process_transaction(&mut client_accounts, transaction),
                    None => {
                        continue;
                    }
                };
            }
            Err(err) => {
                error!(
                    "Rejecting transaction. Unable to read transaction from CSV. Error: {}",
                    err
                );
            }
        }
    }

    let mut writer = csv::Writer::from_writer(stdout);
    for val in client_accounts.values_mut() {
        val.total = val.available + val.held;
        writer.serialize(val).unwrap();
    }

    Ok(())
}
