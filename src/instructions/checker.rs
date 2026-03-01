use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
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
        _token_program,
        _system_program,
        _associated_token_program @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let (amount, bump) = {
        let fundraiser_data = unsafe { fundrasier.borrow_unchecked() };
        let fundraiser_state = wincode::deserialize::<Fundraiser>(&fundraiser_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        let maker_ata_state = TokenAccount::from_account_view(&maker_ata)?;

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
            fundraiser_state.current_amount >= fundraiser_state.amount_to_raise,
            "target not met"
        );

        // check time??

        (fundraiser_state.current_amount, fundraiser_state.bump)
    };

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

    CloseAccount {
        account: vault,
        destination: maker,
        authority: fundrasier,
    }
    .invoke_signed(&[Signer::from(&seed[..])])?;

    Ok(())
}
