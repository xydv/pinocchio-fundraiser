use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_log::log;
use pinocchio_token::{
    instructions::{CloseAccount, Transfer},
    state::TokenAccount,
};

use crate::state::fundraiser::Fundraiser;

pub fn process_checker_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [
        maker,
        mint,
        fundrasier,
        vault,
        maker_ata,
        token_program,
        system_program,
        _associated_token_program @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    log!("1");
    let (amount, bump) = {
        let fundraiser_data = unsafe { fundrasier.borrow_unchecked() };
        let fundraiser_state = wincode::deserialize::<Fundraiser>(&fundraiser_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        if maker_ata.is_data_empty() && maker_ata.lamports() == 0 {
            pinocchio_associated_token_account::instructions::Create {
                funding_account: maker,
                account: maker_ata,
                wallet: maker,
                mint: mint,
                token_program: token_program,
                system_program: system_program,
            }
            .invoke()?;
        }

        let maker_ata_state = TokenAccount::from_account_view(&maker_ata)?;
        let vault_state = TokenAccount::from_account_view(&vault)?;

        if !maker.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if maker_ata_state.owner() != maker.address() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if maker_ata_state.mint() != mint.address() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if fundraiser_state.mint_to_raise != *mint.address().as_array() {
            return Err(ProgramError::InvalidInstructionData);
        }

        if fundraiser_state.maker != *maker.address().as_array() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        assert!(
            vault_state.amount().to_le_bytes() >= fundraiser_state.amount_to_raise,
            "target not met"
        );

        (fundraiser_state.amount_to_raise, fundraiser_state.bump)
    };

    log!("amount {}", u64::from_le_bytes(amount));

    log!("2");
    let bump = [bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];

    Transfer {
        from: vault,
        to: maker_ata,
        authority: fundrasier,
        amount: u64::from_le_bytes(amount),
    }
    .invoke_signed(&[Signer::from(&seed[..])])?;

    // we can close vault ata
    // CloseAccount {
    //     account: vault,
    //     destination: maker,
    //     authority: fundrasier,
    // }
    // .invoke_signed(&[Signer::from(&seed[..])])?;

    // close fundraiser pda
    let lamports = fundrasier.lamports();
    maker.set_lamports(maker.lamports() + lamports);
    fundrasier.set_lamports(0);
    fundrasier.close()?;

    Ok(())
}
