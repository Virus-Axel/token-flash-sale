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
        &[b"deposit", flash_sale_settings.item_name.as_ref(), token_mint.key(), token_deposit_pda.key()],
        &crate::id(),
    );

    let deposit_binding = [expected_deposit_account.1];
    let deposit_seeds = seeds!(
        b"deposit",
        flash_sale_settings.item_name.as_bytes(),
        token_mint.key(),
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
