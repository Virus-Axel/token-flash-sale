use pinocchio::cpi::invoke;
use pinocchio::instruction::{AccountMeta, Instruction, Signer};
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::Sysvar;
use shank::{ShankAccount, ShankInstruction, ShankType};

use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::{find_program_address, Pubkey};
use pinocchio::{seeds, ProgramResult};

use pinocchio::sysvars::clock::Clock;

use solana_program::pubkey::Pubkey as SPK;

#[derive(Clone, ShankAccount)]
pub struct FlashSale {
    pub item_name: String,
    pub price: u64,
    pub init_timestamp: i64,
    pub mint_address: Pubkey,
    pub owner_address: Pubkey,
}

const MAX_NAME_LENGTH: usize = 32;
const FLASH_SALE_ACCOUNT_SIZE: usize = 4 + MAX_NAME_LENGTH + 8 + 8 + 32 + 32;

impl TryFrom<&[u8]> for FlashSale {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut offset = 0;
        let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let item_name = data[offset..offset + name_len]
            .to_vec()
            .into_iter()
            .map(|b| b as char)
            .collect::<String>();

        offset += name_len;

        let price = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;
        let init_timestamp = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;
        let mint_address: Pubkey = data[offset..offset + 32].try_into().unwrap();
        offset += 32;
        let owner_address: Pubkey = data[offset..offset + 32].try_into().unwrap();

        Ok(FlashSale {
            item_name,
            price,
            init_timestamp,
            mint_address,
            owner_address,
        })
    }
}

impl FlashSale {
    pub fn write_to_slice(&self, buf: &mut [u8]) -> Result<(), ProgramError> {
        buf[..4].copy_from_slice(&(self.item_name.len() as u32).to_le_bytes());
        let mut offset = 4;
        buf[offset..offset + self.item_name.len()].copy_from_slice(&self.item_name.as_bytes());
        offset += self.item_name.len();

        buf[offset..offset + 8].copy_from_slice(&self.price.to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&self.init_timestamp.to_le_bytes());
        offset += 8;
        buf[offset..offset + 32].copy_from_slice(&self.mint_address);
        offset += 32;
        buf[offset..offset + 32].copy_from_slice(&self.owner_address);

        Ok(())
    }
}

#[derive(Debug, Clone, ShankType)]
pub struct InitFlashSaleArgs {
    pub initial_price: u64,
    pub sale_duration: u64,
    pub amount: u64,
    pub item_name: String,
}

impl TryFrom<&[u8]> for InitFlashSaleArgs {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let initial_price = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let sale_duration = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let amount = u64::from_le_bytes(data[16..24].try_into().unwrap());

        let name_len = u32::from_le_bytes(data[24..28].try_into().unwrap()) as usize;

        let item_name = data[28..28 + name_len]
            .to_vec()
            .into_iter()
            .map(|b| b as char)
            .collect::<String>();

        Ok(InitFlashSaleArgs {
            initial_price,
            sale_duration,
            amount,
            item_name,
        })
    }
}

#[derive(Debug, Clone, ShankInstruction)]
#[rustfmt::skip]
pub enum InitInstruction {
    #[account(0, writable, signer, name="owner", desc="Owner of the flash sale")]
    #[account(1, writable, name="token_mint", desc="Token mint address of the item being sold")]
    #[account(2, writable, name="source_token_account", desc="Token account with tokens to supply for the sale")]
    #[account(3, writable, name="token_deposit_pda", desc="Account to hold tokens for the sale. Seeds = [\"deposit\", \"item_name\", owner]")]
    #[account(4, writable, name="token_deposit_ata", desc="Associated token account for token_deposit_pda.")]
    #[account(5, writable, name="flash_sale_pda", desc="Account to hold flash sale config and state. Seeds = [\"sale\", \"item_name\", owner]")]
    #[account(6, name="system_program", desc = "System program.")]
    #[account(7, name="token_program", desc = "Token program")]
    #[account(8, name="associated_token_program", desc = "Assosiated token program")]
    #[account(9, name="Sysvar Clock", desc = "Sysvar Clock")]
    #[account(10, name="Sysvar Rent", desc = "Sysvar Rent")]

    InitInstruction(InitFlashSaleArgs),
}

pub fn init_flash_sale(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    
    let owner = accounts_iter.next().unwrap();
    let token_mint = accounts_iter.next().unwrap();
    let source_token_account = accounts_iter.next().unwrap();
    let token_deposit_pda = accounts_iter.next().unwrap();
    let token_deposit_ata = accounts_iter.next().unwrap();
    let flash_sale_pda = accounts_iter.next().unwrap();
    let system_program = accounts_iter.next().unwrap();
    let token_program = accounts_iter.next().unwrap();

    let args = InitFlashSaleArgs::try_from(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let expected_deposit_account = find_program_address(
        &[b"deposit", args.item_name.as_ref(), owner.key()],
        &crate::id(),
    );

    let deposit_binding = [expected_deposit_account.1];
    let stake_seeds = seeds!(
        b"deposit",
        args.item_name.as_bytes(),
        owner.key(),
        &deposit_binding
    );

    pinocchio_system::instructions::Transfer {
        from: owner,
        to: token_deposit_pda,
        lamports: 10_000_000,
    }
    .invoke()?;

    pinocchio_associated_token_account::instructions::Create {
        funding_account: owner,
        account: token_deposit_ata,
        wallet: token_deposit_pda,
        mint: token_mint,
        system_program,
        token_program,
    }
    .invoke_signed(&[Signer::from(&stake_seeds)])?;

    let clock = Clock::get().unwrap();

    let rent = Rent::get().unwrap();
    let minimum_balance = rent.minimum_balance(FLASH_SALE_ACCOUNT_SIZE as usize);

    let expected_sale_account = find_program_address(
        &[b"sale", args.item_name.as_ref(), owner.key()],
        &crate::id(),
    );

    let sale_binding = [expected_sale_account.1];
    let sale_seeds = seeds!(
        b"sale",
        args.item_name.as_bytes(),
        owner.key(),
        &sale_binding
    );

    pinocchio_system::instructions::CreateAccount {
        from: owner,
        to: flash_sale_pda,
        space: FLASH_SALE_ACCOUNT_SIZE as u64,
        lamports: minimum_balance,
        owner: &crate::id(),
    }
    .invoke_signed(&[Signer::from(&sale_seeds)])?;

    let mut flash_sale_data = flash_sale_pda.try_borrow_mut_data()?;

    FlashSale {
        item_name: args.item_name.clone(),
        price: args.initial_price,
        init_timestamp: clock.unix_timestamp,
        mint_address: *token_mint.key(),
        owner_address: *owner.key(),
    }
    .write_to_slice(&mut flash_sale_data)?;

    let stake_ix = spl_token_2022::instruction::transfer_checked(
        &SPK::new_from_array(*token_program.key()),
        &SPK::new_from_array(*source_token_account.key()),
        &SPK::new_from_array(*token_mint.key()),
        &SPK::new_from_array(*token_deposit_ata.key()),
        &SPK::new_from_array(*owner.key()),
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

    invoke(
        &new_ix,
        &[
            source_token_account,
            token_mint,
            token_deposit_ata,
            owner,
            token_program,
        ],
    )?;
    Ok(())
}
