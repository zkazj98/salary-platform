use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, Token,CloseAccount,transfer}, token::TokenAccount};

declare_id!("3efaHuNJ3Aff6MbQ2MgN3BHz1r8kSsWKoSieDL9svbBX");

const USDT_MINT_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
#[program]
pub mod salary_platform {
    use anchor_spl::token::{self, Transfer};

    use super::*;

    pub fn deposit(ctx:Context<Deposit>,mount:u64,unlock_time:i64,secret_key:String)->Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        escrow_account.from = ctx.accounts.sender.key();
        escrow_account.to = ctx.accounts.receiver.key();
        escrow_account.unlock_time = unlock_time;
        escrow_account.mount = mount;
        escrow_account.is_extract = false;
        escrow_account.lock = false;

        let cpi_accounts = Transfer{
            from:ctx.accounts.send_token_account.to_account_info(),
            to:ctx.accounts.escrow_token_account.to_account_info(),
            authority:ctx.accounts.sender.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        transfer(CpiContext::new(cpi_program, cpi_accounts),mount)?;
        emit!(DepositEvent{
            sender:ctx.accounts.sender.key(),
            amount:mount,
            timestamp:unlock_time,
            secret_key:secret_key,
            receiver:ctx.accounts.receiver.key()
        });
        Ok(())
    }

    pub fn withdraw(ctx:Context<Withdraw>,_secret_key:String)->Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        let authority = escrow_account.to_account_info();
        let close_authority = escrow_account.to_account_info();
        let current_time = Clock::get()?.unix_timestamp;
        require!(escrow_account.unlock_time > current_time,EscrowError::UnlockTimeNotReached);
        let amount = escrow_account.mount;
        
        let cpi_accounts = Transfer{
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.receiver_token_account.to_account_info(),
            authority,
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        transfer(CpiContext::new(cpi_program, cpi_accounts),amount)?;
        
        let close_cpi_accounts = CloseAccount{
            account:ctx.accounts.receiver_token_account.to_account_info(),
            destination:ctx.accounts.receiver.to_account_info(),
            authority:close_authority,
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, close_cpi_accounts);
        token::close_account(cpi_ctx)?;

        escrow_account.is_extract = true;
        Ok(())
    }
}
#[account]
pub struct EscrowAccount {
    pub from:Pubkey,
    pub to:Pubkey,
    pub unlock_time:i64,
    pub mount:u64,
    pub is_extract:bool,
    pub lock:bool
}

#[derive(Accounts)]
#[instruction(mount:u64,unlock_time:u64,secret_key:String)]
pub struct Deposit<'info>{

    #[account(mut)]
    pub sender:Signer<'info>,

    pub receiver: AccountInfo<'info>,

    #[account(mut,constraint=send_token_account.mint == usdc_mint.key())]
    pub send_token_account:Account<'info,TokenAccount>,

    #[account(
        init,
        payer=sender,
        space=8+32+32+8+8+1+1,
        seeds=[b"escrow",receiver.key().as_ref()],
        bump,
        constraint=mount>0
    )]
    pub escrow_account:Account<'info,EscrowAccount>,

    #[account(
        init,
        payer=sender,
        seeds=[b"escrow",secret_key.as_bytes(),escrow_account.key().as_ref()],
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

#[derive(Accounts)]
#[instruction(secret_key:String)]
pub struct Withdraw<'info>{
    #[account(mut)]
    pub receiver: Signer<'info>,

    #[account(
        mut,
        seeds = [b"escrow",secret_key.as_bytes(), receiver.key().as_ref()],
        bump,
        constraint=escrow_account.to == receiver.key(),
        close = receiver
    )]
    pub escrow_account: Account<'info,EscrowAccount>,

    #[account(
        mut,
        seeds = [b"escrow",secret_key.as_bytes(), escrow_account.key().as_ref()],
        bump
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub receiver_token_account: Account<'info,TokenAccount>,

    pub token_program: Program<'info,Token>,
}

#[event]
pub struct DepositEvent {
    pub sender: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub secret_key: String,
    pub receiver:Pubkey
}

#[error_code]
pub enum EscrowError {
    #[msg("Unlock time not reached.")]
    UnlockTimeNotReached,

    #[msg("Invalid mint")]
    InvalidMint,

    #[msg("Invalid owner")]
    InvalidOwner,
}

