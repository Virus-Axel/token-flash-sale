pub mod init_flash_sale;
pub mod get_token;

use pinocchio::{
  account_info::AccountInfo, default_allocator, default_panic_handler, entrypoint, msg, program_entrypoint, program_error::ProgramError, pubkey::Pubkey, ProgramResult, MAX_TX_ACCOUNTS
};

use pinocchio_pubkey::declare_id;

declare_id!("96Dq3cwtPC7G8genqLeLKcwVHtxvCxwEFbGLRgLnNZQ8");

program_entrypoint!(process_instruction, MAX_TX_ACCOUNTS);
default_allocator!();

pub fn process_instruction(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  instruction_data: &[u8],
) -> ProgramResult {
  match instruction_data[0]{
    0 => return init_flash_sale::init_flash_sale(accounts, &instruction_data[1..]),
    1 => return get_token::get_token(accounts, &instruction_data[1..]),
    _ => return Err(ProgramError::InvalidInstructionData),
  };
}