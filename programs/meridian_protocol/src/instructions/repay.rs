use anchor_lang::prelude::*;

// use crate::constants::{GOLD_USD_PRICE_FEED, MAX_AGE};
use crate::errors::Errors;
use crate::states::{LendingPool, LoanState, MockOracleState};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use mpl_core::instructions::TransferV1CpiBuilder;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

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

impl<'info> Repay<'info> {
    pub fn repay(&mut self, amount_to_repay: u64) -> Result<()> {
        //e Loan can be repaid only if it's not repaid earlier or your asset is not liquidated
        require!(
            self.borrower_state.loan_status == 0,
            Errors::CannotRepayLoan
        );

        let total_debt_to_repay = self.total_debt_to_repay()?;
        require!(
            amount_to_repay >= total_debt_to_repay,
            Errors::RepayAmountNotEnough
        );

        //User transferring amount to the lending pool protocol
        let token_program = self.token_program.to_account_info();
        let accounts = TransferChecked {
            authority: self.borrower.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            from: self.borrower_usdc_ata.to_account_info(),
            to: self.lending_pool_usdc_ata.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(token_program, accounts);

        transfer_checked(cpi_ctx, amount_to_repay, self.mint_usdc.decimals)?;

        self.borrower_state.loan_status = 1; //repaid
        self.borrower_state.principal_borrowed = 0;
        self.borrower_state.origination_fee = 0;
        self.borrower_state.outstanding_debt = 0;

        //Transferring asset back to user
        self.transfer_asset_to_user()?;

        Ok(())
    }

    fn transfer_asset_to_user(&mut self) -> Result<()> {
        let key = self.lending_pool.key();
        let bump = self.lending_pool.bump_lending_pool;

        let signer_seeds: &[&[u8]] = &[b"meridian_pool", key.as_ref(), &[bump]];
        let seeds = &[signer_seeds];

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .payer(&self.borrower.to_account_info())
            .new_owner(&self.borrower.to_account_info())
            .asset(&self.rwa_asset.to_account_info())
            .authority(Some(&self.lending_pool.to_account_info()))
            .invoke_signed(seeds)?;

        Ok(())
    }

    pub fn total_debt_to_repay(&mut self) -> Result<u64> {
        //e collateral + interest accrued + liquidation_penalty_if applied
        let total_interest_accrued_by_user = self.calculate_interest_accrued()?;
        let principal_borrowed = self.borrower_state.principal_borrowed;
        let origination_fee = self.borrower_state.origination_fee;

        let base_debt = total_interest_accrued_by_user + principal_borrowed + origination_fee;

        let health_factor = self.calculate_health_factor(base_debt)?;

        let total_debt_to_repay: u64;

        if health_factor < 1 {
            let liquidation_penalty = self.calculate_liquidation_penalty()?;
            total_debt_to_repay = liquidation_penalty + base_debt;
        } else {
            total_debt_to_repay = base_debt;
        }

        self.borrower_state.interest_accrued += total_interest_accrued_by_user;
        msg!("Total debt to repay is: {}", total_debt_to_repay);
        Ok(total_debt_to_repay)
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

    pub fn calculate_liquidation_penalty(&mut self) -> Result<u64> {
        let total_user_debt = self.borrower_state.principal_borrowed;

        let liquidation_penalty = total_user_debt
            .checked_mul(self.lending_pool.liquidation_penalty_bps as u64)
            .unwrap()
            .checked_div(10_000)
            .unwrap();

        Ok(liquidation_penalty)
    }

    pub fn calculate_health_factor(&mut self, total_debt: u64) -> Result<u64> {
        //Below Implementation is unsafe, latter is the better one...
        // let health_factor = (self.lending_pool.liquidation_threshold_bps as u64
        //     * self.borrower_state.collateral_value_usd)
        //     / (total_debt.checked_mul(10_000).unwrap());

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

    //withdraw the protocol fees to whatever account..Add a admin controlled function for that!
}
