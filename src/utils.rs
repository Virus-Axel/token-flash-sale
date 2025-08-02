use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

pub fn check_owner(account: &AccountInfo, expected_owner: Pubkey) -> Result<(), ProgramError>{
    match *account.owner() == expected_owner{
        true => Ok(()),
        false => Err(ProgramError::IllegalOwner),
    }
}