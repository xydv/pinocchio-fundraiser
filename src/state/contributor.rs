use pinocchio::{AccountView, error::ProgramError};
use wincode::SchemaRead;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, SchemaRead)]
pub struct Contributor {
    pub amount: [u8; 8],
    pub bump: u8,
}

impl Contributor {
    const LEN: usize = std::mem::size_of::<Contributor>();

    pub fn from_account_info(account_info: &AccountView) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut()?;
        if data.len() != Contributor::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }
}
