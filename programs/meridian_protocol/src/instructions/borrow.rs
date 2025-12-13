use crate::constants::{GOLD_USD_PRICE_FEE, MAX_AGE};
use crate::errors::Errors;
use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
};
use mpl_core::instructions::TransferV1CpiBuilder;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(mut)]
    pub mint_usdc: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"meridian_pool",lending_pool.owner.as_ref()],
        bump = lending_pool.bump_lending_pool
    )]
    pub lending_pool: Box<Account<'info, LendingPool>>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = lending_pool,
        associated_token::token_program = token_program,
    )]
    pub lending_pool_usdc_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = borrower,
        space = 8 + LoanState::INIT_SPACE,
        seeds = [b"meridian_borrower_state", borrower.key().as_ref()],
        bump
    )]
    pub borrower_state: Box<Account<'info, LoanState>>,
    #[account(
        init_if_needed,
        payer = borrower,
        associated_token::mint = mint_usdc,
        associated_token::authority = borrower,
        associated_token::token_program = token_program,
    )]
    pub borrower_usdc_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    ///CHECK: Safe,Will be created on the client side
    #[account(mut)]
    pub rwa_asset: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"meridian_verification_vault", lending_pool.key().as_ref()],
        bump = lending_pool.bump_verification_vault
    )]
    pub protocol_verification_vault: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub price_update: Account<'info, PriceUpdateV2>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    ///CHECK: SAFE
    pub mpl_core_program: AccountInfo<'info>,
}

impl<'info> Borrow<'info> {
    pub fn deposit_for_verification(&mut self) -> Result<()> {
        require!(
            self.borrower_state.is_sent_for_verification == false,
            Errors::AssetAlreadySentForVerification
        );
        require!(self.lending_pool.is_locked == false, Errors::PoolLocked);

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .payer(&self.borrower.to_account_info())
            .asset(&self.rwa_asset.to_account_info())
            .new_owner(&self.protocol_verification_vault)
            .invoke()?;

        self.borrower_state.verification_id += 1;
        self.borrower_state.is_sent_for_verification = true;

        msg!(
            "Asset transferred for verification: {}",
            &self.rwa_asset.key()
        );
        msg!(
            "Your borrower verification id is: {}",
            self.borrower_state.verification_id
        );
        Ok(())
    }

    pub fn borrow(&mut self) -> Result<()> {
        self.borrow_assets()?;
        Ok(())
    }

    pub fn deposit_collateral(&mut self) -> Result<()> {
        require!(
            self.borrower_state.is_verified == true,
            Errors::AssetNotVerified
        );

        let key = self.lending_pool.key();
        let bump = &[self.lending_pool.bump_verification_vault];
        let signer_seeds: &[&[u8]] = 
        &[
            b"meridian_verification_vault",
            key.as_ref(),
            bump            
        ];

        let seeds = &[signer_seeds];

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
        .payer(&self.borrower.to_account_info())
        .asset(&self.rwa_asset.to_account_info())
        .new_owner(&self.lending_pool.to_account_info())
        .authority(Some(&self.protocol_verification_vault.to_account_info()))
        .invoke_signed(seeds)?;

        msg!("Collateral Deposited: {}",self.rwa_asset.key());

        Ok(())
    }

    fn borrow_assets(&mut self) -> Result<()> {
        let borrowable_value = self.calculate_borrowable_value_of_the_asset()?;
        let origination_fee= self.calculate_origination_fee(borrowable_value)?;

        let accounts = TransferChecked{
            from: self.lending_pool_usdc_ata.to_account_info(),
            to: self.borrower_usdc_ata.to_account_info(),
            authority: self.borrower.to_account_info(),
            mint: self.mint_usdc.to_account_info()
        };

        let program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(program,accounts);
        transfer_checked(cpi_ctx, borrowable_value, self.mint_usdc.decimals)?;
        self.borrower_state.principal_borrowed = borrowable_value;
        self.borrower_state.origination_fee += origination_fee;
        Ok(())
    }

    pub fn calculate_origination_fee(&mut self, total_value_borrowed: u64) -> Result<(u64)> {
        return Ok(total_value_borrowed*self.lending_pool.origination_fee_bps as u64/10_000);
        
    }

    pub fn calculate_borrowable_value_of_the_asset(&mut self) -> Result<u64> {
        let price_update_account = &mut self.price_update;

        let gold_usdc_feed_id = get_feed_id_from_hex(GOLD_USD_PRICE_FEE)?;

        let clock = &Clock::get()?;
        let gold_price_latest = price_update_account
            .get_price_no_older_than(clock, MAX_AGE, &gold_usdc_feed_id)?
            .price;

        let weight_in_grams = self.borrower_state.weight_in_grams;
        let purity_in_bps = self.borrower_state.purity_in_bps;
        let ltv = self.lending_pool.loan_to_value_bps;
        let price_of_the_collateral = weight_in_grams
            .checked_mul(purity_in_bps as u64)
            .unwrap()
            .checked_mul(ltv as u64)
            .unwrap()
            .checked_mul(gold_price_latest as u64)
            .unwrap()
            .checked_div(10000)
            .unwrap()
            .checked_div(10000).unwrap();

        Ok(price_of_the_collateral)
    }
}
