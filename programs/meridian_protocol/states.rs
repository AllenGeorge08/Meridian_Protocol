use anchor_lang::prelude::*;


#[account]
pub struct LendingPool{
    pub owner: Pubkey,  //AUTHORITY/OWNER OF THE LENDING POOL
    pub total_deposited_usdc: u64,
    pub total_borrowed: u64,
    pub loan_to_value_bps: u16,  //LOAN TO VALUE (LTV)
    pub protocol_admins: Box<Vec<Pubkey>>,  //PROTOCOL ADMINS
    pub withdrawal_epoch: u64,    //WITHDRAWAL EPOCH PERIOD
    pub lp_total_supply: u64,   //TOTAL LP SUPPLY/SHARES MINTED
    pub usdc_mint: Pubkey,  //MINT FOR THE USDC
    pub protocol_usdc_vault: Pubkey,  //VAULT FOR THE PROTOCOL USDC TREASURY
    
    //BUMP
    pub lending_pool_bump: u8,
    
    //LIQUIDATION 
    pub liquidation_threshold_bps: u16,
    pub liquidation_penalty_bps: u16,
    pub liquidator_reward_bps: u16,

    //UTILIZATION RATE TIERS
    pub utilization_rate_tier_1_bps: u16,
    pub utilization_rate_tier_2_bps: u16,
    pub utilization_rate_tier_3_bps: u16,
    pub utilization_rate_tier_4_bps: u16,
    pub utilization_rate_tier_5_bps: u16,
    
    //APR TIERS
    pub apr_tier_1_bps: u16,
    pub apr_tier_2_bps: u16,
    pub apr_tier_3_bps: u16,
    pub apr_tier_4_bps: u16,
    pub apr_tier_5_bps: u16,

    //FEES
    pub early_withdrawal_fee_bps: u16, //EARLY WITHDRAWAL FEE FOR THE LENDER (5%)..
    pub origination_fee_bps: u16,  //ORIGINATION FEE  (1% for the borrowers)..

    //POOL STATE
    pub is_locked: bool, //IN CASE OF AN EMERGENCY POOL CAN BE LOCKED BY THE ADMIN...

   //PROTOCOL TREASURIES
   pub collateral_escrow: Pubkey,
   pub collateral_verification_escrow: Pubkey,
   pub protocol_fee_vault: Pubkey,
}       


#[account]
pub struct LoanState{
    pub borrower: Pubkey,
    pub nft_mint: Pubkey,
    pub principal_borrowed: u64,
    pub interest_accrued: u64,
    pub outstanding_debt: u64,
    pub borrowed_at: u64,
    pub last_interest_accrued: u64,
    pub collateral_value_usd: u64,
    pub status: u8 , //STATUS = 0(Active), 1(REPAID), 2 (LIQUIDATABLE), 3(LIQUIDATED)
    pub bump_loan_state: u8
}


#[account]
pub struct Lender{
    pub owner: Pubkey,
    pub lp_shares: u64,
    pub deposited_at: u64,
    pub total_deposited: u64,
    pub total_interest_accrued: u64,
    pub bump: u8,
}