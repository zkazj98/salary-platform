use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, Token}, token::TokenAccount};

declare_id!("3efaHuNJ3Aff6MbQ2MgN3BHz1r8kSsWKoSieDL9svbBX");

const USDT_MINT_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
#[program]
pub mod salary_platform {
    use super::*;

}

#[account]
pub struct EscrowAccount {
    pub from:Pubkey,
    pub to:Pubkey,
    pub unlock_time:u64,
    pub mount:u64,
    pub is_extract:bool,
    pub lock:bool
}

#[derive(Accounts)]
#[instruction(mount:u64,unlock_time:u64)]
pub struct Deposit<'info>{

    #[account(mut)]
    pub sender:Signer<'info>,

    #[account(mut,constraint=send_token_account.mint == usdc_mint.key())]
    pub send_token_account:Account<'info,TokenAccount>,

    #[account(
        init,
        payer=sender,
        space=8+32+32+8+8+1+1,
        constraint=mount>0
    )]
    pub escrow_account:Account<'info,EscrowAccount>,

    #[account(
        init,
        payer=sender,
        seeds=[b"escrow",escrow_account.key().as_ref()],
        bump,
        token::mint=usdc_mint,
        token::authority=escrow_account
    )]
    pub escrow_token_account:Account<'info,TokenAccount>,

    #[account(
        constraint = usdc_mint.key() == Pubkey::from_str(USDT_MINT_ADDRESS).unwrap()
    )]
    pub usdc_mint:Account<'info,Mint>,

    pub token_program:Program<'info,Token>,

    pub system_program:Program<'info,System>

}
