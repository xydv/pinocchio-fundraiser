use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, rent::Rent},
};
use pinocchio_log::log;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::state::TokenAccount;
use wincode::SchemaRead;

use crate::{
    helpers::add_le_bytes,
    state::{contributor::Contributor, fundraiser::Fundraiser},
};

#[derive(SchemaRead)]
struct ContributeData {
    amount: [u8; 8],
    bump: u8,
}

pub fn process_contribute_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [
        user,
        mint,
        fundraiser,
        contributor,
        contributor_ata,
        vault,
        system_program,
        token_program,
        _extra_accounts @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let ix_data = wincode::deserialize::<ContributeData>(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    log!("{}", ix_data.bump);
    // log!("{}", ix_data.amount_to_raise);
    log!("{}", u64::from_le_bytes(ix_data.amount));

    // check ata owner and mint
    {
        let contributor_ata_state = TokenAccount::from_account_view(contributor_ata)?;
        if contributor_ata_state.owner() != user.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if contributor_ata_state.mint() != mint.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    log!("here 1");
    // TODO: add other time and contribution checks

    let bump = [ix_data.bump];
    let seeds = [
        Seed::from(b"contributor"),
        Seed::from(fundraiser.address().as_ref()),
        Seed::from(user.address().as_ref()),
        Seed::from(&bump),
    ];

    log!("here 2");
    unsafe {
        log!("here 2.1");
        if contributor.owner() != &crate::ID {
            CreateAccount {
                from: user,
                to: contributor,
                lamports: Rent::get()?.try_minimum_balance(Fundraiser::LEN)?,
                space: Contributor::LEN as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&[Signer::from(&seeds)])?;
        }

        log!("here 2.0");
        let fundraiser_state = Fundraiser::from_account_info(fundraiser)?;
        log!("here 2.0.1");
        let contributor_state = Contributor::from_account_info(contributor)?;

        log!("here 2.2");
        fundraiser_state.current_amount =
            add_le_bytes(fundraiser_state.current_amount, ix_data.amount);
        contributor_state.amount = add_le_bytes(contributor_state.amount, ix_data.amount);
    }

    log!("here 3");
    pinocchio_token::instructions::Transfer {
        from: contributor_ata,
        to: vault,
        authority: user,
        amount: u64::from_le_bytes(ix_data.amount),
    }
    .invoke()?;

    Ok(())
}
