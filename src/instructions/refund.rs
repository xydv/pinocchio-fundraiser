use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_token::instructions::Transfer;

use crate::state::{contributor::Contributor, fundraiser::Fundraiser};

pub fn process_refund(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [
        user,
        maker,
        mint,
        fundraiser,
        contributor,
        contributor_ata,
        vault,
        _token_program,
        _system_program @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let fundraiser_state = Fundraiser::from_account_info(&fundraiser)?;
    let contributor_state = Contributor::from_account_info(&contributor)?;

    let bump = [fundraiser_state.bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];

    Transfer {
        from: vault,
        to: contributor_ata,
        authority: fundraiser,
        amount: u64::from_le_bytes(contributor_state.amount),
    }
    .invoke_signed(&[Signer::from(&seed[..])])?;

    // close the contributor pda
    let lamports = contributor.lamports();
    user.set_lamports(user.lamports() + lamports);
    contributor.set_lamports(0);
    contributor.resize(0)?;

    Ok(())
}
