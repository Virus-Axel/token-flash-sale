use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

pub fn check_owner(account: &AccountInfo, expected_owner: Pubkey) -> Result<(), ProgramError>{
    match *account.owner() == expected_owner{
        true => Ok(()),
        false => Err(ProgramError::IllegalOwner),
    }
}

pub fn check_address(account: &AccountInfo, expected_address: Pubkey) -> Result<(), ProgramError>{
    match *account.key() == expected_address{
        true => Ok(()),
        false => Err(ProgramError::IllegalOwner),
    }
}

pub fn check_address_is_any(account: &AccountInfo, allowed_addresses: &[Pubkey]) -> Result<(), ProgramError>{
    if allowed_addresses.iter().any(|addr| addr == account.key()) {
        Ok(())
    } else {
        Err(ProgramError::IllegalOwner)
    }
}