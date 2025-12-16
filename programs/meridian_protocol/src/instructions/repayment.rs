use anchor_lang::prelude::*;

use crate::constants::{GOLD_USD_PRICE_FEED, MAX_AGE};
use crate::errors::Errors;
use crate::states::{LendingPool, LoanState, MockOracleState};
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
pub struct Repay<'info> {
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
        mut,
        seeds = [b"meridian_borrower_state", borrower.key().as_ref()],
        bump = borrower_state.bump_borrower_state
    )]
    pub borrower_state: Box<Account<'info, LoanState>>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = borrower,
        associated_token::token_program = token_program,
    )]
    pub borrower_usdc_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    ///CHECK: Safe,Will be created on the client side
    #[account(mut)]
    pub rwa_asset: UncheckedAccount<'info>,
    ///CHECK:
    #[account(
        mut,
        seeds = [b"meridian_verification_vault", lending_pool.key().as_ref()],
        bump = lending_pool.bump_verification_vault
    )]
    pub protocol_verification_vault: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"meridian_mock_oracle",lending_pool.key().as_ref()],
        bump = mock_oracle.bump
    )]
    pub mock_oracle: Box<Account<'info, MockOracleState>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub price_update: Account<'info, PriceUpdateV2>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    ///CHECK: SAFE
    pub mpl_core_program: AccountInfo<'info>,
}

impl<'info> Repay<'info>{
    pub fn repay(&mut self) -> Result<()>{
        Ok(())
    }

    pub fn total_debt_to_repay(&mut self) -> Result<u64>{

        Ok(8)
    }

    pub fn calculate_interest_accrued(&mut self) -> Result<()>{
        Ok(())
    }

    pub fn calculate_borrow_rate_tier(&mut self) -> Result<u16>{        
        let current_utilization_rate = self.get_current_utilization_rate()?;
        let current_utilization_rate_bps = current_utilization_rate.checked_mul(10_000).unwrap();
        let u1_bps = self.lending_pool.utilization_rate_tier_1_bps;
        let u2_bps = self.lending_pool.utilization_rate_tier_2_bps;
        let u3_bps = self.lending_pool.utilization_rate_tier_3_bps;
        let u4_bps = self.lending_pool.utilization_rate_tier_4_bps;
        let u5_bps  = self.lending_pool.utilization_rate_tier_5_bps;
   
        Ok(8)


    }

    pub fn get_current_utilization_rate(&mut self) -> Result<u64>{
        let total_borrowed = self.lending_pool.total_borrowed;
        let total_deposited = self.lending_pool.total_deposited_usdc;

        let utilization_rate = total_borrowed.checked_mul(total_deposited).unwrap();

        Ok(utilization_rate)

    }

    pub fn calculate_liquidation_penalty(&mut self) -> Result<u64>{
        let total_user_debt = self.total_debt_to_repay()?;

        let liquidation_penalty = total_user_debt.checked_mul(self.lending_pool.liquidation_penalty_bps as u64).unwrap().checked_div(10_000).unwrap();

        Ok(liquidation_penalty)
    }

    pub fn calculate_health_factor(&mut self) -> Result<u64>{
        let total_debt = self.total_debt_to_repay()?;
        let health_factor = (self.lending_pool.liquidation_threshold_bps as u64* self.borrower_state.collateral_value_usd)/(total_debt.checked_mul(10_000).unwrap());
        
        Ok(health_factor)
    }    
}