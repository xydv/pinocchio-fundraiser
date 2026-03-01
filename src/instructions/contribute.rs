use pinocchio::{AccountView, ProgramResult, error::ProgramError};
use wincode::SchemaRead;

use crate::state::{contributor::Contributor, fundraiser::Fundraiser};

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

    let fundraiser_state = Fundraiser::from_account_info(fundraiser)?;
    let contributor_state = Contributor::from_account_info(contributor)?;

    let amount = u64::from_le_bytes(ix_data.amount);

    pinocchio_token::instructions::Transfer {
        from: contributor_ata,
        to: vault,
        authority: contributor,
        amount,
    }
    .invoke()?;

    Ok(())
}
