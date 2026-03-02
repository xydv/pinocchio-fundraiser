#[cfg(test)]
mod tests {
    use litesvm::LiteSVM;
    use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo, spl_token};
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_associated_token_account::get_associated_token_address;
    use std::path::PathBuf;

    const PROGRAM_ID: &str = "7dTBf2CHGabKL715FsRHyJqjQxVsMWVLYL51FknB1FKf";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;

    fn program_id() -> Pubkey {
        PROGRAM_ID.parse().unwrap()
    }

    fn setup() -> (LiteSVM, Keypair) {
        let mut svm = LiteSVM::new();
        let maker = Keypair::new();

        svm.airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        let so_path = PathBuf::from("target/deploy/pinocchio_fundraiser.so");
        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        svm.add_program(program_id(), &program_data)
            .expect("Failed to add program");

        (svm, maker)
    }

    #[test]
    pub fn test_initialize_instruction() {
        let (mut svm, maker) = setup();
        let program_id = program_id();

        let mint = CreateMint::new(&mut svm, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();

        let (fundraiser_pda, bump) =
            Pubkey::find_program_address(&[b"fundraiser", maker.pubkey().as_ref()], &program_id);

        let vault = get_associated_token_address(&fundraiser_pda, &mint);

        let system_program = solana_sdk_ids::system_program::ID;
        let associated_token_program = spl_associated_token_account::id();

        let amount_to_raise: u64 = 500_000_000;
        let duration: u8 = 30;

        let mut ix_data = Vec::new();
        ix_data.push(0u8);
        ix_data.push(bump);
        ix_data.extend_from_slice(&amount_to_raise.to_le_bytes());
        ix_data.push(duration);

        let initialize_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser_pda, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(associated_token_program, false),
            ],
            data: ix_data,
        };

        let message = Message::new(&[initialize_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&maker], message, recent_blockhash);

        let tx_result = svm.send_transaction(transaction);

        match tx_result {
            Ok(tx) => {
                println!("\nInitialize transaction successful!");
                println!("CUs Consumed: {}", tx.compute_units_consumed);
            }
            Err(e) => panic!("Transaction failed: {:#?}", e),
        }
    }

    #[test]
    pub fn test_contribute_instruction() {
        let (mut svm, maker) = setup();

        let user = Keypair::new();
        svm.airdrop(&user.pubkey(), 5 * LAMPORTS_PER_SOL).unwrap();

        let program_id = program_id();
        let mint = CreateMint::new(&mut svm, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();

        let (fundraiser_pda, bump) =
            Pubkey::find_program_address(&[b"fundraiser", maker.pubkey().as_ref()], &program_id);

        let vault = get_associated_token_address(&fundraiser_pda, &mint);

        let system_program = solana_sdk_ids::system_program::ID;
        let associated_token_program = spl_associated_token_account::id();

        let amount_to_raise: u64 = 500_000_000;
        let duration: u8 = 30;

        let mut ix_data = Vec::new();
        ix_data.push(0u8);
        ix_data.push(bump);
        ix_data.extend_from_slice(&amount_to_raise.to_le_bytes());
        ix_data.push(duration);

        let initialize_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser_pda, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(associated_token_program, false),
            ],
            data: ix_data,
        };

        let message = Message::new(&[initialize_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&maker], message, recent_blockhash);

        svm.send_transaction(transaction).unwrap();

        // contribute
        let user_ata = CreateAssociatedTokenAccount::new(&mut svm, &user, &mint)
            .owner(&user.pubkey())
            .send()
            .unwrap();

        MintTo::new(&mut svm, &maker, &mint, &user_ata, 500_000_000)
            .send()
            .unwrap();

        let (contributor_pda, c_bump) = Pubkey::find_program_address(
            &[
                b"contributor",
                fundraiser_pda.as_ref(),
                user.pubkey().as_ref(),
            ],
            &program_id,
        );

        let contribute_amount: u64 = 200_000_000;

        let mut ix_data = Vec::new();
        ix_data.push(1u8);
        ix_data.extend_from_slice(&contribute_amount.to_le_bytes());
        ix_data.push(c_bump);

        let contribute_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(user.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser_pda, false),
                AccountMeta::new(contributor_pda, false),
                AccountMeta::new(user_ata, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
                AccountMeta::new_readonly(spl_token::ID, false),
            ],
            data: ix_data,
        };

        let message = Message::new(&[contribute_ix], Some(&user.pubkey()));
        let tx = svm
            .send_transaction(Transaction::new(&[&user], message, svm.latest_blockhash()))
            .unwrap();

        println!("\nContribute transaction successful");
        println!("CUs Consumed: {}", tx.compute_units_consumed);
    }
}
