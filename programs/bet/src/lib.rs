use anchor_lang::prelude::*;

declare_id!("6fYXqWh7p9drrq1j9JSRsi7WnU4i6mC57jNm9R5Xmkck");

#[program]
pub mod bet {
    use std::any::Any;

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

        msg!("Bet {:?} created, bet_amount:{:?}", 
        seed, amount);

        Ok(())
    }

    pub fn accept_bet(ctx: Context<AcceptBet>, bet_id:Pubkey, amount: u64, user_public_key: Pubkey, isHeads:bool) -> Result<()> { 
        let bet = &mut ctx.accounts.bet;
        // Verificar que la sala no haya sido resuelta aún
        if !bet.open {
            return err!(Errors::BetOpenedAlready);
        }
        // Verificar que el lado contrario esté vacío
        if (isHeads && bet.heads.is_some()) || (!isHeads && bet.tails.is_some()) {
            return err!(Errors::InvalidBetHeads);
        }

        // Asignar el apostador
        if isHeads {
            bet.heads = Some(ctx.accounts.payer.key());
        } else {
            bet.tails = Some(ctx.accounts.payer.key());
        }

        // Cerrar la apuesta 
        bet.open = false;

        // Transferir el monto
        let cpi_program = ctx.accounts.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: ctx.accounts.payer.to_account_info(),
            to: bet.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, bet.amount)?;

        msg!(
            "Bet {:?} accepted by {:?}, total amount:{:?}",
            bet_id,
            ctx.accounts.payer.key(),
            bet.amount * 2 // Doble de la cantidad apostada
        ); 
        
        Ok(())
    }
    pub fn resolve_bet(ctx: Context<ResolveBet>, seed: u128, isHeads: bool) -> Result<()> {
        let bet = &mut ctx.accounts.bet; 

        // Verificar que la apuesta no esté abierta
        if bet.open {
            return err!(Errors::BetOpenedAlready);
        }

        // Determinar el ganador
        let winner = if isHeads {
            bet.heads.ok_or(ProgramError::InvalidAccountData)?
        } else {
            bet.tails.ok_or(ProgramError::InvalidAccountData)?
        };

        // Transferir el monto total apostado al ganador
        let cpi_program = ctx.accounts.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: bet.to_account_info(),
            to:  ctx.accounts.payer.to_account_info()
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, bet.amount * 2)?;

        msg!(
            "Bet {:?} resolved, winner is {:?}, winning amount:{:?}",
            seed,
            winner,
            bet.amount * 2
        );

        Ok(())
    }

    pub fn close_bet(ctx: Context<CloseBet>, seed: u128) -> Result<()> {
        let bet = &mut ctx.accounts.bet;

        // Verificar si la apuesta está abierta
        if !bet.open {
            return Err(ProgramError::InvalidArgument.into());
        }

        // Verificar que la cuenta que cierra la apuesta sea la que creó la apuesta (heads o tails)
        let is_owner = if let Some(heads) = bet.heads {
            heads == ctx.accounts.payer.key()
        } else if let Some(tails) = bet.tails {
            tails == ctx.accounts.payer.key()
        } else {
            false
        };

        if !is_owner {
            return Err(ProgramError::InvalidArgument.into());
        }

        // Transferir el monto de vuelta al creador de la apuesta
        let cpi_program = ctx.accounts.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: bet.to_account_info(),
            to: ctx.accounts.payer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, bet.amount)?;

        // Marcar la apuesta como cerrada
        bet.open = false;

        msg!("Bet {:?} closed by {:?}, refunded amount:{:?}", seed, ctx.accounts.payer.key(), bet.amount);

        Ok(())
    }

    
    /* 
    pub fn close_bet(ctx: Context<CloseBet> , bet_id:Pubkey) -> Result<()> {
        let bet = &mut ctx.accounts.bet;

        if !bet.open {
            return err!(Errors::BetClosedAlready); 
        }

        /*if bet_id.key().to_string().is_empty() {
            return err!(Errors::InvalidBetId);
        }
        */
        bet.open = true;
        Ok(())
    }
    */
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    #[account(init,
        payer = payer,
        space = 8 + Bet::INIT_SPACE,
        seeds = [b"bet".as_ref()],
        bump,
    )]
    pub bet: Account<'info, Bet>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(seed: u128)]
pub struct CreateBet<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + Bet::INIT_SPACE,
        seeds = [b"apuesta".as_ref(), payer.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub bet: Account<'info, Bet>,

    #[account(mut)]
    user: Account<'info, User>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptBet <'info> {
    #[account(mut)]
    payer: Signer<'info>,  
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResolveBet <'info> {
    #[account(mut)]
    payer: Signer<'info>,  
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseBet  <'info> {
    #[account(mut)]
    payer: Signer<'info>,  
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
struct Bet {
    heads: Option<Pubkey>,
    tails: Option<Pubkey>,
    amount: u64,
    open: bool,
    resolver: Pubkey
}

#[account]
#[derive(InitSpace)]
struct User {
    pub owner: Pubkey,
}


#[error_code]
pub enum Errors {
    #[msg("Cannot create a new bet")]
    CannotCreateBet, 
    #[msg("Bet Amount cannot be empty")]
    BetAmountEmpty, 
    #[msg("Bet is already empty")]
    BetEmpty, 
    #[msg("Invalid Bet Id")]
    InvalidBetId, 
    #[msg("Invalid Bet Heads")]
    InvalidBetHeads, 
    #[msg("User is not part fo this bet")]
    InvalidUserBet, 
    #[msg("Bet is already closed")]
    BetClosedAlready, 
    #[msg("Bet is already opened")]
    BetOpenedAlready, 
    #[msg("Insufficient funds to place the bet.")]
    InsufficientFunds,
    #[msg("Cannot join your own room.")]
    CannotJoinOwnRoom,

    
}
 