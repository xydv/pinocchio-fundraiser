use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock, rent::Rent},
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::state::TokenAccount;
use wincode::SchemaRead;

use crate::{
    constants::{MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER, SECONDS_TO_DAYS},
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
        _system_program,
        _token_program,
        _extra_accounts @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let ix_data = wincode::deserialize::<ContributeData>(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let amount = u64::from_le_bytes(ix_data.amount);
    let mint_state = pinocchio_token::state::Mint::from_account_view(mint)?;

    assert!(
        amount > 1_u8.pow(mint_state.decimals() as u32) as u64,
        "contribution too small"
    );

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

    // TODO: add other time and contribution checks

    let bump = [ix_data.bump];
    let seeds = [
        Seed::from(b"contributor"),
        Seed::from(fundraiser.address().as_ref()),
        Seed::from(user.address().as_ref()),
        Seed::from(&bump),
    ];

    unsafe {
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

        let fundraiser_state = Fundraiser::from_account_info(fundraiser)?;

        let contributor_state = Contributor::from_account_info(contributor)?;

        assert!(
            amount
                <= (u64::from_le_bytes(fundraiser_state.amount_to_raise)
                    * MAX_CONTRIBUTION_PERCENTAGE)
                    / PERCENTAGE_SCALER,
            "contribution too big"
        );

        assert!(
            (u64::from_le_bytes(contributor_state.amount)
                <= (u64::from_le_bytes(fundraiser_state.amount_to_raise)
                    * MAX_CONTRIBUTION_PERCENTAGE)
                    / PERCENTAGE_SCALER)
                && (u64::from_le_bytes(contributor_state.amount) + amount
                    <= (u64::from_le_bytes(fundraiser_state.amount_to_raise)
                        * MAX_CONTRIBUTION_PERCENTAGE)
                        / PERCENTAGE_SCALER),
            "maximum contribution reached"
        );

        let current_time = Clock::get()?.unix_timestamp;

        assert!(
            fundraiser_state.duration
                > ((current_time - i64::from_le_bytes(fundraiser_state.time_started))
                    / SECONDS_TO_DAYS) as u8,
            "fundraising ended"
        );

        fundraiser_state.current_amount =
            add_le_bytes(fundraiser_state.current_amount, ix_data.amount);
        contributor_state.amount = add_le_bytes(contributor_state.amount, ix_data.amount);
    }

    pinocchio_token::instructions::Transfer {
        from: contributor_ata,
        to: vault,
        authority: user,
        amount: u64::from_le_bytes(ix_data.amount),
    }
    .invoke()?;

    Ok(())
}
