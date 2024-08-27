use anchor_lang::prelude::*;

declare_id!("6fYXqWh7p9drrq1j9JSRsi7WnU4i6mC57jNm9R5Xmkck");

use anchor_lang::system_program::{transfer, Transfer};
use anchor_lang::solana_program::{program::invoke};

#[program]
pub mod bet {
    use std::any::Any;
    use anchor_lang::solana_program::{program::invoke, system_instruction::transfer};

    

    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;

        Ok(())
    }
    
    pub fn create_bet(ctx: Context<CreateBet>, seed: u128, amount: u64,  resolver:Pubkey,  is_heads:bool) -> Result<()> {
        ctx.accounts.create_bet(seed,amount,resolver,is_heads)?;

        Ok(())
    }

    pub fn accept_bet(ctx: Context<AcceptBet>, bet_id:Pubkey, user_public_key: Pubkey, is_heads:bool) -> Result<()> { 
        ctx.accounts.accept_bet(bet_id,user_public_key,is_heads)?;
        
        Ok(())
    }
    pub fn resolve_bet(ctx: Context<ResolveBet>, seed: u128, resolver:Pubkey, is_heads:bool) -> Result<( )> {
        ctx.accounts.resolve_bet(seed ,resolver, is_heads )?;

        Ok(())
    } 
    pub fn close_bet(ctx: Context<CloseBet>) -> Result<()> { 
        ctx.accounts.close_bet()?;
        Ok(())
    } 

 
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

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.bet = self.bet.clone(); 
        Ok(())
    }  
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
    pub system_program: Program<'info, System>,
} 
impl<'info> CreateBet<'info> {
    pub fn create_bet(&mut self, seed: u128,amount: u64, resolver: Pubkey, is_heads: bool) -> Result<()> {
        
        let bet = &mut self.bet;
        bet.set_inner(Bet {
            heads: if is_heads {Some(self.payer.key())}else{None},
            tails: if !is_heads {Some(self.payer.key())}else{None},
            amount,
            open: true,
            resolver: resolver
        });

        // Native sol transfer
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.payer.to_account_info(),
            to: self.bet.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, amount)?;
         
        /*
        let from_account = &self.payer;
        let to_account = &self.bet;
        // Create the transfer instruction
        let transfer_instruction = system_instruction::transfer(from_account.key, &to_account.key(), amount);

         anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                from_account.to_account_info(),
                to_account.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &[], //seeds, pero como el signer == payer == from, no es necesario agregar la firma
        )?;
        */
        msg!("Bet {:?} created, bet_amount:{:?}", 
        seed , amount);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct AcceptBet <'info> {
    #[account(mut)]
    payer: Signer<'info>,  
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    pub system_program: Program<'info, System>,
} 
impl<'info> AcceptBet<'info> {  
    pub fn accept_bet(&mut self, bet_id:Pubkey, user_public_key: Pubkey, is_heads:bool) -> Result<()> { 
        let bet = &mut self.bet;
        // Verificar que la sala no haya sido resuelta aún
        if !bet.open {
            return err!(Errors::BetOpenedAlready);
        }
        // Verificar que el lado contrario esté vacío
        if (is_heads && bet.heads.is_some()) || (!is_heads && bet.tails.is_some()) {
            return err!(Errors::InvalidBetHeads);
        }

        // Asignar el apostador
        if is_heads {
            bet.heads = Some(user_public_key);
        } else {
            bet.tails = Some(user_public_key);
        }

        // Cerrar la apuesta 
        bet.open = false;

        // Transferir el monto
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.payer.to_account_info(),
            to: bet.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, bet.amount)?;

        msg!(
            "Bet {:?} accepted by {:?}, total amount:{:?}",
            bet_id,
            self.payer.key(),
            bet.amount * 2 // Doble de la cantidad apostada
        ); 
            
        Ok(())
    }
}

#[derive(Accounts)]
pub struct ResolveBet <'info> {
    #[account(mut)]
    payer: Signer<'info>,  
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    pub system_program: Program<'info, System>,
}
impl<'info> ResolveBet<'info> {
    pub fn resolve_bet(&mut self, seed: u128, resolver:Pubkey, is_heads:bool) -> Result<()> {
        let bet = &mut self.bet; 
        //msg!("Resolver {:?} has the result is: {}", resolver, is_heads);
        
        // Verificar que la apuesta no esté abierta
        if !bet.open {
            return err!(Errors::BetOpenedAlready);
        }

        // Verificar que ambas partes de la apuesta estén ocupadas
        if bet.heads.is_none() || bet.tails.is_none() {
            return err!(Errors::IncompleteBet);
        }

        // Verificar que el `resolver` sea una de las partes de la apuesta
        if resolver != bet.heads.unwrap() && resolver != bet.tails.unwrap() {
            return err!(Errors::UnauthorizedResolver);
        }

        // Determinar el ganador  basado en el parámetro `is_heads`
        let winner = if is_heads {
            bet.heads.ok_or(ProgramError::InvalidAccountData)?
        } else {
            bet.tails.ok_or(ProgramError::InvalidAccountData)?
        };
        /*
        let winner = if is_heads {
            bet.heads.unwrap()
        } else {
            bet.tails.unwrap()
        }; 
        */

        // Marcar la apuesta como resuelta y almacenar el ganador
        bet.open = false;
        //bet.winner = Some(winner);

        msg!(
            "Bet resolved by {:?}, result is: {:?}, winner is {:?}",
            resolver,
            if is_heads { "Heads" } else { "Tails" },
            winner
        );
 

        // Transferir el monto total apostado al ganador

        // Invoke the transfer instruction 
        // from: room (PDA) to: payer
        /* 
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                bet.to_account_info(),
                self.payer.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &[], 
        )?;
        */ 

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: bet.to_account_info(),
            to:  self.payer.to_account_info()
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
}
#[derive(Accounts)]
pub struct CloseBet  <'info> {
    #[account(mut)]
    payer: Signer<'info>,  
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    pub system_program: Program<'info, System>,
}
impl<'info> CloseBet<'info> { 
    pub fn close_bet(&mut self) -> Result<()> { 
        let bet = &mut self.bet;

        // Verificar que la apuesta esté abierta
        if !bet.open {
            return err!(Errors::BetClosedAlready);
        }
        // Verificar que el usuario sea el creador de la apuesta
        let is_owner = bet.heads == Some(self.payer.key()) || bet.tails == Some(self.payer.key());
        if !is_owner {
            return err!(Errors::UnauthorizedCancellation);
        }

        // Verificar que la cuenta que cancela la apuesta sea la que creó la apuesta (heads o tails)
        let is_owner = if let Some(heads) = bet.heads {
            heads == self.payer.key()
        } else if let Some(tails) = bet.tails {
            tails == self.payer.key()
        } else {
            false
        };

         // Invoke the transfer instruction 
        // from: room (PDA) to: payer
        /*
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                bet.to_account_info(),
                self.payer.to_account_info(),
                self.system_program.to_account_info(),
                bet.amount
            ],
            &[], 
            
        )?;
        */

        // Transferir el monto de vuelta al creador de la apuesta
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: bet.to_account_info(),
            to: self.payer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, bet.amount)?;

        // Restablecer el estado de la apuesta
        bet.open = false;
        bet.heads = None;
        bet.tails = None;

        // Marcar la apuesta como cerrada
        bet.open = false;

        /*
        msg!(
            "Bet {:?} cancelled by {:?}, amount refunded:{:?}",
            bet_id,
            self.payer.key(),
            bet.amount
        );
        */

        msg!(
            "Bet cancelled by {:?}, amount refunded:{:?}", 
            self.payer.key(),
            bet.amount
        );
   
        Ok(())
    }
}
 

#[account]
#[derive(InitSpace)]
pub struct Bet {
    pub heads: Option<Pubkey>,
    pub tails: Option<Pubkey>,
    pub amount: u64,
    pub open: bool,
    pub resolver: Pubkey
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
    #[msg("Unauthorized Cancellation.")]
    UnauthorizedCancellation,
    #[msg("Incomplete Bet.")]
    IncompleteBet,
    #[msg("Unauthorized Resolver.")]
    UnauthorizedResolver
    
}
 