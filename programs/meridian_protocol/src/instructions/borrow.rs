use crate::constants::{GOLD_USD_PRICE_FEED, MAX_AGE};
use crate::errors::Errors;
use crate::states::{LendingPool, LoanState, MockOracleState};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
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

impl<'info> Borrow<'info> {
    pub fn deposit_for_verification(&mut self, bumps: &BorrowBumps) -> Result<()> {
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
        self.borrower_state.bump_borrower_state = bumps.borrower_state;

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

    pub fn is_asset_verified(&mut self) -> bool {
        if self.borrower_state.is_verified {
            println!("Your asset is verified");
            return true;
        } else {
            println!("Your asset is not verified")
        }
        return false;
    }

    //e WE'RE USING MOCK ORACLE AS FOR NOW...
    pub fn borrow(&mut self, use_pyth: bool) -> Result<()> {
        require!(self.lending_pool.is_locked == false, Errors::PoolLocked);
        require!(self.is_asset_verified(), Errors::AssetNotVerified);

        self.borrower_state.last_interest_accrued = Clock::get()?.unix_timestamp;

        self.borrow_assets(use_pyth)?;
        self.borrower_state.loan_status = 0;
        Ok(())
    }

    pub fn deposit_collateral(&mut self) -> Result<()> {
        require!(
            self.borrower_state.is_verified == true,
            Errors::AssetNotVerified
        );

        let key = self.lending_pool.key();
        let bump = &[self.lending_pool.bump_verification_vault];
        let signer_seeds: &[&[u8]] = &[b"meridian_verification_vault", key.as_ref(), bump];

        let seeds = &[signer_seeds];

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .payer(&self.borrower.to_account_info())
            .asset(&self.rwa_asset.to_account_info())
            .new_owner(&self.lending_pool.to_account_info())
            .authority(Some(&self.protocol_verification_vault.to_account_info()))
            .invoke_signed(seeds)?;

        msg!("Collateral Deposited: {}", self.rwa_asset.key());

        Ok(())
    }

    //e Borrower can collect their collateral back if the collateral is rejected by the admin
    pub fn collect_collateral(&mut self) -> Result<()> {
        require!(
            self.borrower_state.is_verified == false && self.borrower_state.is_rejected,
            Errors::CannotCollectCollateral
        );

        let key = self.lending_pool.key();
        let bump = &[self.lending_pool.bump_verification_vault];
        let signer_seeds: &[&[u8]] = &[b"meridian_verification_vault", key.as_ref(), bump];

        let seeds = &[signer_seeds];

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .payer(&self.borrower.to_account_info())
            .asset(&self.rwa_asset.to_account_info())
            .new_owner(&self.borrower.to_account_info())
            .authority(Some(&self.protocol_verification_vault.to_account_info()))
            .invoke_signed(seeds)?;

        msg!("Collateral Collected Back: {}", self.rwa_asset.key());

        Ok(())
    }

    fn borrow_assets(&mut self, use_pyth: bool) -> Result<()> {
        let borrowable_value: u64;
        if use_pyth {
            borrowable_value = self.calculate_borrowable_value_of_the_asset_pyth()?;
        } else {
            borrowable_value = self.calculate_borrowable_value_of_the_asset_mock_oracle()?;
        }

        let origination_fee = self.calculate_origination_fee(borrowable_value)?;

        require!(
            self.lending_pool.total_deposited_usdc >= borrowable_value,
            Errors::InsufficientLiquidityToBorrow
        );

        let owner = self.lending_pool.owner.key();

        //e Signer seeds
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"meridian_pool",
            owner.as_ref(),
            &[self.lending_pool.bump_lending_pool],
        ]];

        let accounts = TransferChecked {
            mint: self.mint_usdc.to_account_info(),
            from: self.lending_pool_usdc_ata.to_account_info(),
            to: self.borrower_usdc_ata.to_account_info(),
            authority: self.lending_pool.to_account_info(),
        };

        let program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(program, accounts, signer_seeds);
        transfer_checked(cpi_ctx, borrowable_value, self.mint_usdc.decimals)?;
        self.borrower_state.principal_borrowed = borrowable_value;
        self.borrower_state.origination_fee += origination_fee;
        self.lending_pool.total_borrowed += borrowable_value;
        self.borrower_state.borrow_apr_bps = self.calculate_borrow_rate_tier()?;
        self.borrower_state.collateral_value_usd =
            self.calculate_value_of_the_asset_mock_oracle()?;
        Ok(())
    }

    pub fn calculate_value_of_the_asset_mock_oracle(&mut self) -> Result<u64> {
        let mock_oracle = &mut self.mock_oracle;

        let max_age = 100;
        let gold_price_per_gram_scaled = mock_oracle.get_price_per_gram(max_age)?;
        //e Price of the collateral = weight in grams * Purity of the gold(in bps) * Gold price latest(In grams)
        let weight_in_grams = self.borrower_state.weight_in_grams;
        let purity_in_bps = self.borrower_state.purity_in_bps;

        let price_of_the_collateral = (weight_in_grams as u64)
            .checked_mul(gold_price_per_gram_scaled as u64)
            .unwrap()
            .checked_mul(purity_in_bps as u64)
            .unwrap()
            .checked_div(10_000) //e to normalize purity bps
            .unwrap()
            .checked_div(1_000_000)
            .unwrap();

        Ok(price_of_the_collateral)
    }

    pub fn calculate_borrow_rate_tier(&mut self) -> Result<u16> {
        //e Retrieving current utlization rate...
        let current_utilization_rate = self.get_current_utilization_rate()?;
        let current_utilization_rate_bps = current_utilization_rate as u16;
        // let current_utilization_rate_bps =
        //     current_utilization_rate.checked_div(10_000).unwrap() as u16;
        let u1_bps = self.lending_pool.utilization_rate_tier_1_bps; //0 to 2500
        let u2_bps = self.lending_pool.utilization_rate_tier_2_bps; //2500 to //5000
        let u3_bps = self.lending_pool.utilization_rate_tier_3_bps; //5000 to 7500
        let u4_bps = self.lending_pool.utilization_rate_tier_4_bps; //7500 to //9000
        let u5_bps = self.lending_pool.utilization_rate_tier_5_bps; //9000+

        let current_borrow_apr_rate_bps: u16;

        if current_utilization_rate_bps >= u1_bps && current_utilization_rate_bps < u2_bps {
            current_borrow_apr_rate_bps = self.lending_pool.apr_tier_1_bps;
        } else if current_utilization_rate_bps >= u2_bps && current_utilization_rate_bps < u3_bps {
            current_borrow_apr_rate_bps = self.lending_pool.apr_tier_2_bps;
        } else if current_utilization_rate_bps >= u3_bps && current_utilization_rate_bps < u4_bps {
            current_borrow_apr_rate_bps = self.lending_pool.apr_tier_3_bps;
        } else if current_utilization_rate_bps >= u4_bps && current_utilization_rate_bps < u5_bps {
            current_borrow_apr_rate_bps = self.lending_pool.apr_tier_4_bps;
        } else {
            current_borrow_apr_rate_bps = self.lending_pool.apr_tier_5_bps
        }

        Ok(current_borrow_apr_rate_bps)
    }

    //e Total_Borrowed * 10_000/Total_Deposited..
    pub fn get_current_utilization_rate(&mut self) -> Result<u64> {
        let total_borrowed = self.lending_pool.total_borrowed;
        let total_deposited = self.lending_pool.total_deposited_usdc;

        if total_deposited == 0 {
            return Ok(0);
        }

        let utilization_rate = total_borrowed
            .checked_mul(10_000)
            .unwrap()
            .checked_div(total_deposited)
            .unwrap();

        Ok(utilization_rate)
    }

    pub fn calculate_origination_fee(&mut self, total_value_borrowed: u64) -> Result<u64> {
        return Ok(total_value_borrowed * self.lending_pool.origination_fee_bps as u64 / 10_000);
    }

    pub fn calculate_borrowable_value_of_the_asset_mock_oracle(&mut self) -> Result<u64> {
        let mock_oracle = &mut self.mock_oracle;

        let max_age = 100;
        let gold_price_per_gram_scaled = mock_oracle.get_price_per_gram(max_age)?;
        //e Price of the collateral = weight in grams * Purity of the gold(in bps) * Gold price latest(In grams)
        let weight_in_grams = self.borrower_state.weight_in_grams;
        let purity_in_bps = self.borrower_state.purity_in_bps;
        let ltv = self.lending_pool.loan_to_value_bps;

        let price_of_the_collateral = (weight_in_grams as u64)
            .checked_mul(gold_price_per_gram_scaled as u64)
            .unwrap()
            .checked_mul(purity_in_bps as u64)
            .unwrap()
            .checked_mul(ltv as u64)
            .unwrap()
            .checked_div(1_00_000) //e to normalized ltv bps
            .unwrap()
            .checked_div(10_000) //e to normalize purity bps
            .unwrap()
            .checked_div(1_000_000)
            .unwrap();

        Ok(price_of_the_collateral)
    }

    pub fn calculate_borrowable_value_of_the_asset_pyth(&mut self) -> Result<u64> {
        let price_update_account = &mut self.price_update;

        let gold_usdc_feed_id = get_feed_id_from_hex(GOLD_USD_PRICE_FEED)?;

        let clock = &Clock::get()?;
        let gold_price =
            price_update_account.get_price_no_older_than(clock, MAX_AGE, &gold_usdc_feed_id)?;

        let gold_price_per_troy_ounce_amount = gold_price.price;

        //Pyth returns value in troy unit = 31.103476 grams
        const GRAMS_PER_TROY_OUNCE_SCALED: i64 = 31_103_476; // Troy ounce -> grams i.e 31.103476 * 10**6 to avoid rounding errors

        let gold_price_per_gram_scaled = gold_price_per_troy_ounce_amount
            .checked_mul(1_000_000)
            .unwrap()
            .checked_div(GRAMS_PER_TROY_OUNCE_SCALED)
            .unwrap();

        //e Price of the collateral = weight in grams * Purity of the gold(in bps) * Gold price latest(In grams)
        let weight_in_grams = self.borrower_state.weight_in_grams;
        let purity_in_bps = self.borrower_state.purity_in_bps;
        let ltv = self.lending_pool.loan_to_value_bps;

        let price_of_the_collateral = (weight_in_grams as u64)
            .checked_mul(gold_price_per_gram_scaled as u64)
            .unwrap()
            .checked_mul(purity_in_bps as u64)
            .unwrap()
            .checked_mul(ltv as u64)
            .unwrap()
            .checked_div(1_00_000)
            .unwrap()
            .checked_div(10_000)
            .unwrap()
            .checked_div(1_000_000)
            .unwrap();

        Ok(price_of_the_collateral)
    }
}
