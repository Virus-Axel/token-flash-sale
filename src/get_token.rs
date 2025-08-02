use pinocchio::{
    account_info::AccountInfo, cpi::invoke_signed, instruction::{AccountMeta, Instruction, Signer}, program_error::ProgramError, pubkey::find_program_address, seeds, ProgramResult
};
use shank::{ShankInstruction, ShankType};

use crate::init_flash_sale::{FlashSale, InitFlashSaleArgs};
use solana_program::pubkey::Pubkey as SPK;

#[derive(Debug, Clone, ShankType)]
pub struct GetTokenArgs {
    pub amount: u64,
}

#[derive(Debug, Clone, ShankInstruction)]
#[rustfmt::skip]
pub enum GetToken {
    #[account(0, writable, signer, name="payer", desc="Payer of the Sol")]
    #[account(1, writable, name="receiver_token_ata", desc="Token account that will receivet the tokens")]
    #[account(2, writable, name="token_mint", desc="Token mint address of the item to get")]
    #[account(3, writable, name="token_deposit_pda", desc="Account that holds tokens for the sale. Seeds = [\"deposit\", \"item_name\", owner]")]
    #[account(4, writable, name="token_deposit_ata", desc="Associated token account for token_deposit_pda.")]
    #[account(5, writable, name="flash_sale_owner", desc="Owner Pubkey of the flash sale owner.")]
    #[account(6, writable, name="flash_sale_pda", desc="Account that holds information about the sale. Seeds = [\"sale\", \"item_name\", owner]")]
    #[account(7, name="system_program", desc = "System program.")]
    #[account(8, name="token_program", desc = "Token program")]
    #[account(9, name="associated_token_program", desc = "Assosiated token program")]
    #[account(10, name="Sysvar Clock", desc = "Sysvar Clock")]
    #[account(11, name="Sysvar Rent", desc = "Sysvar Rent")]

    GetToken(GetTokenArgs),
}

pub fn get_token(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer = accounts_iter.next().unwrap();
    let receiver_token_ata = accounts_iter.next().unwrap();
    let token_mint = accounts_iter.next().unwrap();
    let token_deposit_pda = accounts_iter.next().unwrap();
    let token_deposit_ata = accounts_iter.next().unwrap();
    let flash_sale_owner = accounts_iter.next().unwrap();
    let flash_sale_pda = accounts_iter.next().unwrap();
    let system_program = accounts_iter.next().unwrap();
    let token_program = accounts_iter.next().unwrap();

    let flash_sale_data = flash_sale_pda.try_borrow_mut_data()?;
    let flash_sale_settings = FlashSale::try_from(flash_sale_data.as_ref())
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    pinocchio_system::instructions::Transfer {
        from: payer,
        to: flash_sale_owner,
        lamports: 10_000_000,
    }
    .invoke()?;

    let stake_ix = spl_token_2022::instruction::transfer_checked(
        &SPK::new_from_array(*token_program.key()),
        &SPK::new_from_array(*token_deposit_ata.key()),
        &SPK::new_from_array(*token_mint.key()),
        &SPK::new_from_array(*receiver_token_ata.key()),
        &SPK::new_from_array(*token_deposit_pda.key()),
        &[],
        1,
        9,
    )
    .unwrap();

    let account_metas: Vec<AccountMeta> = stake_ix
        .accounts
        .iter()
        .map(|m| AccountMeta {
            is_signer: m.is_signer,
            is_writable: m.is_writable,
            pubkey: m.pubkey.as_array(),
        })
        .collect();

    let new_ix: Instruction<'_, '_, '_, '_> = Instruction {
        program_id: &stake_ix.program_id.to_bytes(),
        data: &stake_ix.data,
        accounts: &account_metas,
    };

    let expected_deposit_account = find_program_address(
        &[b"deposit", flash_sale_settings.item_name.as_ref(), token_deposit_pda.key()],
        &crate::id(),
    );

    let deposit_binding = [expected_deposit_account.1];
    let deposit_seeds = seeds!(
        b"deposit",
        flash_sale_settings.item_name.as_bytes(),
        token_deposit_pda.key(),
        &deposit_binding
    );

    invoke_signed(
        &new_ix,
        &[
            token_deposit_ata,
            token_mint,
            receiver_token_ata,
            token_deposit_pda,
            token_program,
        ],
        &[Signer::from(&deposit_seeds)]
    )?;
    Ok(())
}
