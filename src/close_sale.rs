use pinocchio::cpi::{invoke, invoke_signed};
use pinocchio::instruction::{AccountMeta, Instruction, Signer};
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::Sysvar;
use pinocchio_system::instructions::{Assign, Transfer};
use shank::{ShankAccount, ShankInstruction, ShankType};

use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::{find_program_address, Pubkey};
use pinocchio::{msg, seeds, ProgramResult};

use pinocchio::sysvars::clock::Clock;

use solana_program::pubkey::Pubkey as SPK;

use crate::init_flash_sale::FlashSale;
use crate::utils::check_owner;

fn deinit_account_if_exists(account: &AccountInfo, receiver: &AccountInfo, signers: &[Signer]) -> ProgramResult {
    let lamports = *account.try_borrow_lamports().unwrap();
    if lamports == 0 {
        return Ok(());
    }

    *account.try_borrow_mut_lamports().unwrap() = 0;
    *receiver.try_borrow_mut_lamports().unwrap() += lamports;

    /*Assign {
        account: account,
        owner: &pinocchio_system::id(),
    }
    .invoke_signed(signers)?;*/

    Ok(())
}

pub fn close_sale(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    
    let owner = accounts_iter.next().unwrap();
    let receiver_token_ata = accounts_iter.next().unwrap();
    let token_mint = accounts_iter.next().unwrap();
    let token_deposit_pda = accounts_iter.next().unwrap();
    let token_deposit_ata = accounts_iter.next().unwrap();
    let flash_sale_pda = accounts_iter.next().unwrap();
    let system_program = accounts_iter.next().unwrap();
    let token_program = accounts_iter.next().unwrap();
    
    let args = {
        let flash_sale_data = flash_sale_pda.try_borrow_data()?;
        FlashSale::try_from(flash_sale_data.as_ref())
            .map_err(|_| ProgramError::InvalidAccountData)?
    };

    if !owner.is_signer(){
        msg!("Owner must be signer");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if args.mint_address != *token_mint.key(){
        msg!("Unexpected token mint address");
        return Err(ProgramError::InvalidArgument);
    }
    if args.owner_address != *owner.key(){
        msg!("Unexpected flash sale owner");
        return Err(ProgramError::InvalidArgument);
    }
    check_owner(flash_sale_pda, crate::id())?;

    let expected_deposit_account = find_program_address(
        &[b"deposit", args.item_name.as_ref(), token_mint.key(), owner.key()],
        &crate::id(),
    );

    let deposit_binding = [expected_deposit_account.1];
    let deposit_seeds = seeds!(
        b"deposit",
        args.item_name.as_bytes(),
        token_mint.key(),
        owner.key(),
        &deposit_binding
    );

    let expected_sale_account = find_program_address(
        &[b"sale", args.item_name.as_ref(), token_mint.key(), owner.key()],
        &crate::id(),
    );

    let sale_binding = [expected_sale_account.1];
    let sale_seeds = seeds!(
        b"sale",
        args.item_name.as_bytes(),
        token_mint.key(),
        owner.key(),
        &sale_binding
    );

    let transfer_tokens_instruction = spl_token_2022::instruction::transfer_checked(
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

    let account_metas: Vec<AccountMeta> = transfer_tokens_instruction
        .accounts
        .iter()
        .map(|m| AccountMeta {
            is_signer: m.is_signer,
            is_writable: m.is_writable,
            pubkey: m.pubkey.as_array(),
        })
        .collect();

    let new_ix: Instruction<'_, '_, '_, '_> = Instruction {
        program_id: &transfer_tokens_instruction.program_id.to_bytes(),
        data: &transfer_tokens_instruction.data,
        accounts: &account_metas,
    };

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

    deinit_account_if_exists(flash_sale_pda, owner, &[Signer::from(&sale_seeds)])?;

    Ok(())
}
