use {
    domichain_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    domichain_program_test::*,
    solana_sdk::{account::Account, signature::Signer, transaction::Transaction},
    spl_example_transfer_satomis::processor::process_instruction,
    std::str::FromStr,
};

#[tokio::test]
async fn test_satomi_transfer() {
    let program_id = Pubkey::from_str("TransferSatomis111111111111111111111111111").unwrap();
    let source_pubkey = Pubkey::new_unique();
    let destination_pubkey = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "spl_example_transfer_satomis",
        program_id,
        processor!(process_instruction),
    );
    program_test.add_account(
        source_pubkey,
        Account {
            satomis: 5,
            owner: program_id, // Can only withdraw satomis from accounts owned by the program
            ..Account::default()
        },
    );
    program_test.add_account(
        destination_pubkey,
        Account {
            satomis: 890_875,
            ..Account::default()
        },
    );
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &(),
            vec![
                AccountMeta::new(source_pubkey, false),
                AccountMeta::new(destination_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}
