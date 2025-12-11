// use anchor_lang::prelude::*;

// use crate::{states::*};

// #[derive(Accounts)]
// pub struct Initialize<'info>{
//     pub authority: Signer,
//     pub admins: Box<Vec<Pubkey>>,
//     #[account(
//         init_if_needed,
//         payer = authority,
//         seeds = [b"meridian_pool",authority.key().as_ref()],
//         bump
//     )] 
//     pub lending_pool_state: Box<Account<'info,LendingPool>>,
//     #[account(
//         init_if_needed,
//         payer = authority,
//         seeds = [b"protocol_seize_vault",authority.key().as_ref()],
//         bump 
//     )]


// }