pub mod close_sale;
pub mod get_token;
pub mod init_flash_sale;
pub mod utils;

use pinocchio::{
    account_info::AccountInfo, default_allocator, default_panic_handler, entrypoint, msg,
    program_entrypoint, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
    MAX_TX_ACCOUNTS,
};

use pinocchio_pubkey::declare_id;
use shank::ShankInstruction;

use crate::{get_token::GetTokenArgs, init_flash_sale::InitFlashSaleArgs};

declare_id!("96Dq3cwtPC7G8genqLeLKcwVHtxvCxwEFbGLRgLnNZQ8");

program_entrypoint!(process_instruction, MAX_TX_ACCOUNTS);
default_allocator!();

#[derive(Debug, Clone, ShankInstruction)]
#[rustfmt::skip]
pub enum CloseSale {
  #[account(0, writable, signer, name="owner", desc="Owner of the flash sale")]
  #[account(1, writable, name="token_mint", desc="Token mint address of the item being sold")]
  #[account(2, writable, name="source_token_account", desc="Token account with tokens to supply for the sale")]
  #[account(3, writable, name="token_deposit_pda", desc="Account to hold tokens for the sale. Seeds = [\"deposit\", \"item_name\", token_mint, owner]")]
  #[account(4, writable, name="token_deposit_ata", desc="Associated token account for token_deposit_pda.")]
  #[account(5, writable, name="flash_sale_pda", desc="Account to hold flash sale config and state. Seeds = [\"sale\", \"item_name\", token_mint, owner]")]
  #[account(6, name="system_program", desc = "System program.")]
  #[account(7, name="token_program", desc = "Token program")]
  #[account(8, name="associated_token_program", desc = "Assosiated token program")]
  #[account(9, name="Sysvar Clock", desc = "Sysvar Clock")]
  #[account(10, name="Sysvar Rent", desc = "Sysvar Rent")]
  InitInstruction(InitFlashSaleArgs),
  
  #[account(0, writable, signer, name="owner", desc="Owner of the flash sale")]
  #[account(1, writable, name="receiver_token_ata", desc="Token account that will receivet the tokens")]
  #[account(2, writable, name="token_mint", desc="Token mint address of the item to get")]
  #[account(3, writable, name="token_deposit_pda", desc="Account to hold tokens for the sale. Seeds = [\"deposit\", \"item_name\", token_mint, owner]")]
  #[account(4, writable, name="token_deposit_ata", desc="Associated token account for token_deposit_pda.")]
  #[account(5, writable, name="flash_sale_pda", desc="Account to hold flash sale config and state. Seeds = [\"sale\", \"item_name\", token_mint, owner]")]
  #[account(6, name="system_program", desc = "System program.")]
  #[account(7, name="token_program", desc = "Token program")]
  #[account(8, name="associated_token_program", desc = "Assosiated token program")]
  #[account(9, name="Sysvar Clock", desc = "Sysvar Clock")]
  #[account(10, name="Sysvar Rent", desc = "Sysvar Rent")]
  CloseSale(GetTokenArgs),

  #[account(0, writable, signer, name="payer", desc="Payer of the Sol")]
  #[account(1, writable, name="receiver_token_ata", desc="Token account that will receivet the tokens")]
  #[account(2, writable, name="token_mint", desc="Token mint address of the item to get")]
  #[account(3, writable, name="token_deposit_pda", desc="Account that holds tokens for the sale. Seeds = [\"deposit\", \"item_name\", token_mint, owner]")]
  #[account(4, writable, name="token_deposit_ata", desc="Associated token account for token_deposit_pda.")]
  #[account(5, writable, name="flash_sale_owner", desc="Owner Pubkey of the flash sale owner.")]
  #[account(6, writable, name="flash_sale_pda", desc="Account that holds information about the sale. Seeds = [\"sale\", \"item_name\", token_mint, owner]")]
  #[account(7, name="system_program", desc = "System program.")]
  #[account(8, name="token_program", desc = "Token program")]
  #[account(9, name="associated_token_program", desc = "Assosiated token program")]
  #[account(10, name="Sysvar Clock", desc = "Sysvar Clock")]
  #[account(11, name="Sysvar Rent", desc = "Sysvar Rent")]

  GetToken(GetTokenArgs),
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data[0] {
        0 => init_flash_sale::init_flash_sale(accounts, &instruction_data[1..]),
        1 => close_sale::close_sale(accounts, &instruction_data[1..]),
        2 => get_token::get_token(accounts, &instruction_data[1..]),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
