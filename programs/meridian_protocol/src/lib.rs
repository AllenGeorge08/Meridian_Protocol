pub mod constants;
pub mod errors;
pub mod instructions;
pub mod states;
use anchor_lang::prelude::*;
pub use instructions::*;
pub use instructions::{mock_oracle};
pub use states::*;

declare_id!("BWECTiw4de85dPk7t9Sems64115EugAvzQ9sFPPi12N2");

#[program]
pub mod meridian_protocol{

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        loan_to_value_bps: u16,
        utilization_rate_tier_1_bps: u16,
        utilization_rate_tier_2_bps: u16,
        utilization_rate_tier_3_bps: u16,
        utilization_rate_tier_4_bps: u16,
        utilization_rate_tier_5_bps: u16,
        apr_tier_1_bps: u16,
        apr_tier_2_bps: u16,
        apr_tier_3_bps: u16,
        apr_tier_4_bps: u16,
        apr_tier_5_bps: u16,
        early_withdrawal_fee_bps: u16,
        origination_fee_bps: u16,
        withdrawal_epoch: i64,
        liquidation_threshold_bps: u16,
        liquidation_penalty_bps: u16,
        liquidator_reward_bps: u16,
    ) -> Result<()> {
        ctx.accounts.initialize_pool(
            loan_to_value_bps,
            utilization_rate_tier_1_bps,
            utilization_rate_tier_2_bps,
            utilization_rate_tier_3_bps,
            utilization_rate_tier_4_bps,
            utilization_rate_tier_5_bps,
            apr_tier_1_bps,
            apr_tier_2_bps,
            apr_tier_3_bps,
            apr_tier_4_bps,
            apr_tier_5_bps,
            early_withdrawal_fee_bps,
            origination_fee_bps,
            withdrawal_epoch,
            liquidation_threshold_bps,
            liquidation_penalty_bps,
            liquidator_reward_bps,
            &ctx.bumps,
        )?;
        Ok(())
    }

    pub fn lock(ctx: Context<LockPool>) -> Result<()> {
        ctx.accounts.lock_pool()?;
        Ok(())
    }

    pub fn unlock_pool(ctx: Context<UnLockPool>) -> Result<()> {
        ctx.accounts.unlock_pool()?;
        Ok(())
    }

    pub fn add_admin(ctx: Context<AddAdmin>, admin: Pubkey) -> Result<()> {
        ctx.accounts.add_admin(admin)?;
        Ok(())
    }

    pub fn remove_admin(ctx: Context<RemoveAdmin>, admin: Pubkey) -> Result<()> {
        ctx.accounts.remove_admin(admin)?;
        Ok(())
    }

    pub fn update_collateral_valuation(
        ctx: Context<UpdateCollateralValuation>,
        amount: u64,
    ) -> Result<()> {
        ctx.accounts.update_collateral_valuation(amount)?;
        Ok(())
    }

    pub fn update_oracle_values(ctx: Context<MockOracle>,price: i64,exponent: i32) -> Result<()>{
        ctx.accounts.update_oracle_values(price, exponent)?;
        Ok(())
    }

    pub fn amount_to_shares(ctx: Context<Lending>, deposit_amount: u64) -> Result<u64> {
        let amount = ctx.accounts.calculate_shares_to_mint(deposit_amount);

        msg!(
            "If you deposit {} then you receive {} amount of shares",
            deposit_amount,
            amount
        );
        Ok(amount)
    }

    //LENDER OPERATIONS
    pub fn deposit(ctx: Context<Lending>, deposit_amount: u64) -> Result<()> {
        ctx.accounts.deposit_liquidity(deposit_amount)?;
        Ok(())
    }

    pub fn calculate_withdrawable_amount(ctx: Context<Withdraw>, lp_shares: u64) -> Result<u64> {
        let withdrawable_shares = ctx.accounts.get_total_withdrawable_amount(lp_shares);
        msg!(
            "You can withdraw {} usdc for {} amount of lp shares",
            withdrawable_shares,
            lp_shares
        );
        Ok(withdrawable_shares)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        ctx.accounts.withdraw_liquidity()?;
        msg!(
            "Liquidity Withdrawn by Lender: {}",
            ctx.accounts.lender.key()
        );
        Ok(())
    }

    pub fn get_total_interest_earned(
        ctx: Context<Withdraw>,
        total_deposited_collateral: u64,
        total_lp_shares_owned: u64,
    ) -> Result<()> {
        let earned = ctx
            .accounts
            .get_total_interest_earned(total_deposited_collateral, total_lp_shares_owned);
        msg!("You've earned: {} interest on your collateral", earned);
        Ok(())
    }

    //Borrow functions
    pub fn deposit_collateral_for_verification(ctx: Context<Borrow>) -> Result<()> {
        ctx.accounts.deposit_for_verification(&ctx.bumps)?;
        Ok(())
    }

    //ADMIN VERIFICATION..
    pub fn verify_asset(
        ctx: Context<Verify_asset>,
        verification_id: u32,
        is_verified: bool,
        purity_in_bps: u16,
        weight_in_grams: i64,
    ) -> Result<()> {
        ctx.accounts
            .verify_asset(verification_id, is_verified, purity_in_bps, weight_in_grams)?;
        Ok(())
    }

    //DEPOSIT COLLATERAL TO THE LPOOL..
    pub fn deposit_collateral(ctx: Context<Borrow>) -> Result<()> {
        ctx.accounts.deposit_collateral()?;
        Ok(())
    }

    pub fn borrow_assets(ctx: Context<Borrow>) -> Result<()> {
        ctx.accounts.borrow(false)?;
        Ok(())
    }

    pub fn collect_asset_back(ctx: Context<Borrow>) -> Result<()> {
        ctx.accounts.collect_collateral()?;
        msg!(
            "Borrower: {} collected back his collateral",
            ctx.accounts.borrower.key()
        );
        Ok(())
    }

    //GETTER FUNCTIONS LEFT
    pub fn calculate_borrowable_value_of_your_asset(ctx: Context<Borrow>) -> Result<()> {
        let value = ctx
            .accounts
            .calculate_borrowable_value_of_the_asset_mock_oracle()?;
        msg!("Current Value Of Your Asset Is: {}", value);
        Ok(())
    }

    pub fn is_asset_verified(ctx: Context<Borrow>) -> Result<()> {
        ctx.accounts.is_asset_verified();
        Ok(())
    }

    pub fn get_origination_fee(ctx: Context<Borrow>, total_borrowed: u64) -> Result<()> {
        let origination_fee = ctx.accounts.calculate_origination_fee(total_borrowed)?;
        msg!("Origination fee for your asset: {}", origination_fee);
        Ok(())
    }

    //REPAY
    pub fn repay_debt(ctx: Context<Repay>, amount_to_repay: u64) -> Result<()> {
        ctx.accounts.repay(amount_to_repay)?;
        Ok(())
    }

    //GETTER FUNCTIONS FOR REPAY LEFT
    pub fn total_debt_left(ctx: Context<Repay>) -> Result<()> {
        let total_debt = ctx.accounts.total_debt_to_repay()?;
        
        Ok(())
    }

    //LIQUIDATE
    pub fn liquidate(ctx: Context<Liquidate>) -> Result<()> {
        ctx.accounts.liquidate()?;
        Ok(())
    }
}
