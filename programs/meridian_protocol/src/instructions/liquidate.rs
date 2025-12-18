use crate::errors::Errors;
use crate::states::{LendingPool, LoanState, MockOracleState};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use mpl_core::instructions::TransferV1CpiBuilder;

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,
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
        payer = liquidator,
        associated_token::mint = mint_usdc,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program,
    )]
    pub liquidator_usdc_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [b"meridian_borrower_state", borrower_state.borrower.key().as_ref()],
        bump = borrower_state.bump_borrower_state
    )]
    pub borrower_state: Box<Account<'info, LoanState>>,
    ///CHECK: Safe,Will be created on the client side
    #[account(mut)]
    pub rwa_asset: UncheckedAccount<'info>,
    ///CHECK: Protocol PDA where the liquidation seized collateral rwa will be sent
    #[account(
        mut,
        seeds = [b"meridian_seize_vault", lending_pool.key().as_ref()],
        bump,
    )]
    pub protocol_seize_vault: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"meridian_mock_oracle",lending_pool.key().as_ref()],
        bump = mock_oracle.bump
    )]
    pub mock_oracle: Box<Account<'info, MockOracleState>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    ///CHECK: SAFE
    pub mpl_core_program: AccountInfo<'info>,
}

impl<'info> Liquidate<'info> {
    pub fn liquidate(&mut self) -> Result<()> {
        require!(
            self.borrower_state.loan_status == 0,
            Errors::CannotLiquidate
        );
        let liquidation_penalty = self.calculate_liquidation_penalty()?;
        let (_total_debt_to_repay, health_factor) =
            self.total_debt_to_repay(liquidation_penalty)?; //e Didn't calculate differently to avoid circular dependencies

        require!(health_factor < 1, Errors::CannotLiquidate);

        self.transfer_asset_to_seize_vault()?;
        self.transfer_penalty_shares_to_the_liquidator(liquidation_penalty)?;

        self.borrower_state.loan_status = 2;

        Ok(())
    }

    fn transfer_penalty_shares_to_the_liquidator(&mut self, total_penalty: u64) -> Result<()> {
        let liquidator_reward_bps = self.lending_pool.liquidator_reward_bps;

        let shares_to_transfer = total_penalty
            .checked_mul(liquidator_reward_bps as u64)
            .unwrap()
            .checked_div(10_000)
            .unwrap();

        let accounts = TransferChecked {
            from: self.lending_pool_usdc_ata.to_account_info(),
            to: self.liquidator_usdc_ata.to_account_info(),
            authority: self.lending_pool.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
        };
        let lending_pool_owner = self.lending_pool.owner.key();
        let seeds: &[&[&[u8]]] = &[&[
            b"meridian_pool",
            lending_pool_owner.as_ref(),
            &[self.lending_pool.bump_lending_pool],
        ]];

        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, seeds);
        transfer_checked(cpi_ctx, shares_to_transfer, self.mint_usdc.decimals)?;

        self.lending_pool.total_deposited_usdc -= shares_to_transfer;
        Ok(())
    }

    fn transfer_asset_to_seize_vault(&mut self) -> Result<()> {
        let lending_pool_owner = self.lending_pool.owner.key();
        let seeds: &[&[&[u8]]] = &[&[
            b"meridian_pool",
            lending_pool_owner.as_ref(),
            &[self.lending_pool.bump_lending_pool],
        ]];

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .payer(&self.liquidator.to_account_info())
            .new_owner(&self.protocol_seize_vault.to_account_info())
            .asset(&self.rwa_asset.to_account_info())
            .authority(Some(&self.lending_pool.to_account_info()))
            .invoke_signed(seeds)?;

        Ok(())
    }

    pub fn total_debt_to_repay(&mut self, liquidation_penalty: u64) -> Result<(u64, u64)> {
        //e collateral + interest accrued + liquidation_penalty_if applied
        let total_interest_accrued_by_user = self.calculate_interest_accrued()?;
        let principal_borrowed = self.borrower_state.principal_borrowed;
        let origination_fee = self.borrower_state.origination_fee;

        let base_debt = total_interest_accrued_by_user + principal_borrowed + origination_fee;

        let health_factor = self.calculate_health_factor(base_debt)?;

        let total_debt_to_repay: u64;

        if health_factor < 1 {
            total_debt_to_repay = liquidation_penalty + base_debt;
        } else {
            total_debt_to_repay = base_debt;
        }

        self.borrower_state.interest_accrued += total_interest_accrued_by_user;

        Ok((total_debt_to_repay, health_factor))
    }

    pub fn calculate_interest_accrued(&mut self) -> Result<u64> {
        pub const SECONDS_PER_YEAR: u64 = 31_536_000;
        let current_time = Clock::get()?.unix_timestamp;
        let last_interest_accrued_at = self.borrower_state.last_interest_accrued;

        let borrow_apr_at_the_time_of_borrow = self.borrower_state.borrow_apr_bps;
        let time_delta = current_time - last_interest_accrued_at; //e Won't go below 0 so we can convert it to u64

        let interest_accrued_numerator = self
            .borrower_state
            .principal_borrowed
            .checked_mul(borrow_apr_at_the_time_of_borrow as u64)
            .unwrap()
            .checked_mul(time_delta as u64)
            .unwrap();
        let interest_accrued_denominator = 10000 * SECONDS_PER_YEAR; //e Won't overflow

        let total_interest_accrued = interest_accrued_numerator
            .checked_div(interest_accrued_denominator)
            .unwrap();

        Ok(total_interest_accrued)
    }

    fn calculate_liquidation_penalty(&mut self) -> Result<u64> {
        // let total_user_debt = self.total_debt_to_repay()?;
        let total_user_debt = self.borrower_state.principal_borrowed;

        let liquidation_penalty = total_user_debt
            .checked_mul(self.lending_pool.liquidation_penalty_bps as u64)
            .unwrap()
            .checked_div(10_000)
            .unwrap();

        Ok(liquidation_penalty)
    }

    fn calculate_health_factor(&mut self, total_debt: u64) -> Result<u64> {
        if total_debt == 0 {
            msg!("No total debt, Health factor is infinite");
            return Ok(u64::MAX);
        }

        let ltv = self.lending_pool.liquidation_threshold_bps as u64;
        let collateral_supplied = self.borrower_state.collateral_value_usd;

        let denominator = total_debt.checked_mul(10_000).unwrap();

        let health_factor = ltv
            .checked_mul(collateral_supplied)
            .unwrap()
            .checked_div(denominator)
            .unwrap();

        Ok(health_factor)
    }

    pub fn get_current_health_factor(&mut self) -> Result<u64> {
        let liquidation_penalty = self.calculate_liquidation_penalty()?;
        let (_total_debt_to_repay, health_factor) =
            self.total_debt_to_repay(liquidation_penalty)?;

        msg!("The current health factor is: {}", health_factor);

        Ok(health_factor)
    }
}
