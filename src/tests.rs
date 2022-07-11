#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    #[test]
    fn should_error_on_bad_file() {
        let mut stdout = Vec::new();
        match crate::do_run(
            &String::from("test-data/definitely-does-not-exist.csv"),
            &mut stdout,
        ) {
            Ok(_result) => {
                panic!("This shouldn't happen!")
            }
            Err(_err) => {}
        }
    }

    #[test]
    fn should_deposit_single_transaction_successfully() {
        let mut stdout = Vec::new();
        match crate::do_run(&String::from("test-data/single-deposit.csv"), &mut stdout) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,1.2345,0.0000,1.2345,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_not_be_able_to_withdrawal_single_transaction() {
        let mut stdout = Vec::new();
        match crate::do_run(
            &String::from("test-data/single-withdrawal.csv"),
            &mut stdout,
        ) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,0.0000,0.0000,0.0000,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_be_able_to_deposit_withdraw_simple() {
        let mut stdout = Vec::new();
        match crate::do_run(
            &String::from("test-data/simple-deposit-and-withdrawal.csv"),
            &mut stdout,
        ) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,0.0005,0.0000,0.0005,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_not_be_able_to_withdraw_more_than_available() {
        let mut stdout = Vec::new();
        match crate::do_run(&String::from("test-data/double-withdraw.csv"), &mut stdout) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,0.2345,0.0000,0.2345,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_be_able_to_dispute() {
        let mut stdout = Vec::new();
        match crate::do_run(&String::from("test-data/simple-dispute.csv"), &mut stdout) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,0.0000,1.2345,1.2345,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_not_be_able_to_dispute_while_dispute_in_progress() {
        let mut stdout = Vec::new();
        match crate::do_run(
            &String::from("test-data/dispute-while-dispute-in-progress.csv"),
            &mut stdout,
        ) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,0.0000,1.2345,1.2345,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_not_chargeback_if_transaction_is_not_in_dispute() {
        let mut stdout = Vec::new();
        match crate::do_run(&String::from("test-data/bad-chargeback.csv"), &mut stdout) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,1.2345,0.0000,1.2345,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_not_resolve_if_transaction_is_not_in_dispute() {
        let mut stdout = Vec::new();
        match crate::do_run(&String::from("test-data/bad-resolve.csv"), &mut stdout) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,1.2345,0.0000,1.2345,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_chargeback_if_transaction_is_in_dispute() {
        let mut stdout = Vec::new();
        match crate::do_run(&String::from("test-data/good-chargeback.csv"), &mut stdout) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,0.0000,0.0000,0.0000,true\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_resolve_if_transaction_is_in_dispute() {
        let mut stdout = Vec::new();
        match crate::do_run(&String::from("test-data/good-resolve.csv"), &mut stdout) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,1.2345,0.0000,1.2345,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_not_allow_transaction_if_account_locked() {
        let mut stdout = Vec::new();
        match crate::do_run(
            &String::from("test-data/good-chargeback-with-more-transactions.csv"),
            &mut stdout,
        ) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,0.0000,0.0000,0.0000,true\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_not_be_able_to_dispute_if_resolved() {
        let mut stdout = Vec::new();
        match crate::do_run(
            &String::from("test-data/dispute-after-resolution.csv"),
            &mut stdout,
        ) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,1.2345,0.0000,1.2345,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_ignore_bad_rows_in_csv() {
        let mut stdout = Vec::new();
        match crate::do_run(
            &String::from("test-data/bad-record-ignored.csv"),
            &mut stdout,
        ) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,0.0005,0.0000,0.0005,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_correctly_calculate_total() {
        let mut stdout = Vec::new();
        match crate::do_run(&String::from("test-data/test-total.csv"), &mut stdout) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,1.500,1.2345,2.7345,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }

    #[test]
    fn should_only_allow_one_transaction_id() {
        let mut stdout = Vec::new();
        match crate::do_run(
            &String::from("test-data/double-transactions.csv"),
            &mut stdout,
        ) {
            Ok(_result) => {
                assert_eq!(
                    from_utf8(&stdout).unwrap(),
                    "client,available,held,total,locked\n1,1.2345,0.0000,1.2345,false\n"
                )
            }
            Err(_err) => {
                panic!("This shouldn't happen!")
            }
        }
    }
}
