use anchor_lang::prelude::*;

declare_id!("6fYXqWh7p9drrq1j9JSRsi7WnU4i6mC57jNm9R5Xmkck");

#[program]
pub mod bet {
    use anchor_lang::system_program::{transfer, Transfer};

    use super::*;

    pub fn create_bet(ctx: Context<CreateBet>, seed: u128, amount: u64, resolver:Pubkey, isHeads:bool) -> Result<()> {
        let bet = &mut ctx.accounts.bet;
        bet.set_inner(Bet {
            heads: if isHeads {Some(ctx.accounts.payer.key())}else{None},
            tails: if !isHeads {Some(ctx.accounts.payer.key())}else{None},
            amount,
            open: true,
            resolver
        });

        // Native sol transfer
        let cpi_program = ctx.accounts.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: ctx.accounts.payer.to_account_info(),
            to: ctx.accounts.bet.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, amount)?;

        Ok(())
    }
/*
    pub fn accept_bet(ctx: Context<AcceptBet>) -> Result<()> {
        Ok(())
    }

    pub fn resolve_bet(ctx: Context<ResolveBet>) -> Result<()> {
        Ok(())
    }

    pub fn close_bet(ctx: Context<CloseBet>) -> Result<()> {
        Ok(())
    }
     */
}

#[derive(Accounts)]
#[instruction(seed: u128)]
pub struct CreateBet<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + Bet::INIT_SPACE,
        seeds = [b"apuesta".as_ref(), payer.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump
    )]
    bet: Account<'info, Bet>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptBet {}

#[derive(Accounts)]
pub struct ResolveBet {}

#[derive(Accounts)]
pub struct CloseBet {}

#[account]
#[derive(InitSpace)]
struct Bet {
    heads: Option<Pubkey>,
    tails: Option<Pubkey>,
    amount: u64,
    open: bool,
    resolver: Pubkey
}