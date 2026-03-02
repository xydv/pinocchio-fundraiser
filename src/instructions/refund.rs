use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_log::log;
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::{
    helpers::sub_le_bytes,
    state::{contributor::Contributor, fundraiser::Fundraiser},
};

pub fn process_refund_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
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

    log!("1");

    {
        let contributor_ata_state = TokenAccount::from_account_view(contributor_ata)?;
        if contributor_ata_state.owner() != user.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if contributor_ata_state.mint() != mint.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }
    log!("2");

    let (amount, bump) = {
        let fundraiser_state = Fundraiser::from_account_info(&fundraiser)?;
        let contributor_state = Contributor::from_account_info(&contributor)?;
        let vault_state = TokenAccount::from_account_view(&vault)?;

        assert!(
            vault_state.amount().to_le_bytes() < fundraiser_state.amount_to_raise,
            "target met"
        );

        fundraiser_state.current_amount =
            sub_le_bytes(fundraiser_state.current_amount, contributor_state.amount);

        (contributor_state.amount, fundraiser_state.bump)
    };
    log!("3");

    let bump = [bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];

    Transfer {
        from: vault,
        to: contributor_ata,
        authority: fundraiser,
        amount: u64::from_le_bytes(amount),
    }
    .invoke_signed(&[Signer::from(&seed[..])])?;

    // close the contributor pda
    let lamports = contributor.lamports();
    user.set_lamports(user.lamports() + lamports);
    contributor.set_lamports(0);
    contributor.resize(0)?;

    Ok(())
}
