use anchor_lang::prelude::*;

declare_id!("BRDnpo7F4dRfxQ68BRjztaBd2GqkahZtBpkbZXf2nKtD");


#[account]
#[derive(InitSpace)]
pub struct PendingRequest {
    pub sender: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub id: u64,
}
 
#[derive(Accounts)]
pub struct InitializeRequest<'info> {
    #[account(
        init,
        seeds = [b"pending-request", signer.key().as_ref(), cash_account.request_counter.to_le_bytes().as_ref()],
        bump,
        payer = signer,
        space = 8 + PendingRequest::INIT_SPACE
    )]
    pub pending_request: Account<'info, PendingRequest>,
    #[account(
        mut,
        seeds = [b"cash-account", signer.key().as_ref()],
        bump,
        close = signer,
    )]
    pub cash_account: Account<'info, CashAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
 

#[program]
pub mod cash_app {
    use anchor_lang::solana_program::{program::invoke, system_instruction};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn new_request(ctx: Context<InitializeRequest>, recipient: Pubkey, amount: u64) -> Result<()> {
        let cash_account = &mut ctx.accounts.cash_account;
        let pending_request = &mut ctx.accounts.pending_request;
        pending_request.sender = *ctx.accounts.signer.key;
        pending_request.recipient = recipient;
        pending_request.amount = amount;
        pending_request.id = cash_account.request_counter;
        cash_account.request_counter += 1;

        Ok(())
    }

    pub fn decline_request(_ctx: Context<DeclineRequest>) -> Result<()> {
        Ok(())
    }

    pub fn accept_request(ctx: Context<AcceptRequest>) -> Result<()> {
        let amount = ctx.accounts.pending_request.amount;
 
        let from_cash_account = &mut ctx.accounts.from_cash_account.to_account_info();
        let to_cash_account = &mut ctx.accounts.to_cash_account.to_account_info();
 
        **from_cash_account.try_borrow_mut_lamports()? -= amount;
        **to_cash_account.try_borrow_mut_lamports()? += amount;
 
        Ok(())
    }

    pub fn initialize_account(ctx: Context<InitializeAccount>) -> Result<()> {
        let cash_account = &mut ctx.accounts.cash_account;
        cash_account.owner = *ctx.accounts.signer.key;
        cash_account.friends = Vec::new();
        cash_account.request_counter = 0;
        Ok(())
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);
 
        let ix = system_instruction::transfer(
            &ctx.accounts.signer.key(),
            ctx.accounts.cash_account.to_account_info().key,
            amount,
        );
 
        invoke(
            &ix,
            &[
                ctx.accounts.signer.clone(),
                ctx.accounts.cash_account.to_account_info(),
            ],
        )?;
 
        Ok(())
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);
 
        let cash_account = &mut ctx.accounts.cash_account.to_account_info();
        let wallet = &mut ctx.accounts.signer.to_account_info();
 
        require!(*cash_account.owner == ctx.accounts.signer.key(), ErrorCode::InvalidSigner);
 
        **cash_account.try_borrow_mut_lamports()? -= amount;
        **wallet.try_borrow_mut_lamports()? += amount;
 
        Ok(())
    }

    pub fn transfer_funds(
        ctx: Context<TransferFunds>,
        _recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);
 
        let from_cash_account = &mut ctx.accounts.from_cash_account.to_account_info();
        let to_cash_account = &mut ctx.accounts.to_cash_account.to_account_info();
 
        require!(*from_cash_account.owner == ctx.accounts.signer.key(), ErrorCode::InvalidSigner);
 
        **from_cash_account.try_borrow_mut_lamports()? -= amount;
        **to_cash_account.try_borrow_mut_lamports()? += amount;
 
        Ok(())
    }

    pub fn add_friend(ctx: Context<AddFriend>, pubkey: Pubkey) -> Result<()> {
        let cash_account = &mut ctx.accounts.cash_account;
        cash_account.friends.push(pubkey);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(
        mut,
        seeds = [b"cash-account", signer.key().as_ref()],
        bump,
    )]
    pub cash_account: Account<'info, CashAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
 
 
#[derive(Accounts)]
pub struct InitializeAccount<'info> {
    #[account(
        init,
        seeds = [b"cash-account", signer.key().as_ref()],
        bump,
        payer = signer,
        space = 8 + CashAccount::INIT_SPACE
    )]
    pub cash_account: Account<'info, CashAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Initialize {}

#[account]
#[derive(InitSpace)]
pub struct CashAccount {
    pub owner: Pubkey,
    #[max_len(100)]
    pub friends: Vec<Pubkey>,
    pub request_counter: u64,
}


#[derive(Accounts)]
pub struct DepositFunds<'info> {
    #[account(
        mut,
        seeds = [b"cash-account", signer.key().as_ref()],
        bump,
    )]
    pub cash_account: Account<'info, CashAccount>,
    #[account(mut)]
    /// CHECK: This account is only used to transfer SOL, not for data storage.
    pub signer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(recipient: Pubkey)]
pub struct TransferFunds<'info> {
    #[account(
        mut,
        seeds = [b"cash-account", signer.key().as_ref()],
        bump,
    )]
    pub from_cash_account: Account<'info, CashAccount>,
 
    #[account(
        mut,
        seeds = [b"cash-account", recipient.key().as_ref()],
        bump,
    )]
    pub to_cash_account: Account<'info, CashAccount>,
    pub system_program: Program<'info, System>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct AddFriend<'info> {
    #[account(
        mut,
        seeds = [b"cash-account", signer.key().as_ref()],
        bump,
    )]
    pub cash_account: Account<'info, CashAccount>,
    #[account(mut)]
    /// CHECK: This account is only used to transfer SOL, not for data storage.
    pub signer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DeclineRequest<'info> {
    #[account(
        mut,
        seeds = [b"pending-request", signer.key().as_ref(), pending_request.id.to_le_bytes().as_ref()],
        bump,
        close = signer,
    )]
    pub pending_request: Account<'info, PendingRequest>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptRequest<'info> {
    #[account(
        mut,
        seeds = [b"pending-request", signer.key().as_ref(), pending_request.id.to_le_bytes().as_ref()],
        bump,
        close = signer,
    )]
    pub pending_request: Account<'info, PendingRequest>,
    #[account(
        mut,
        seeds = [b"cash-account", pending_request.sender.key().as_ref()],
        bump,
    )]
    pub from_cash_account: Account<'info, CashAccount>,
    #[account(
        mut,
        seeds = [b"cash-account", pending_request.recipient.key().as_ref()],
        bump,
    )]
    pub to_cash_account: Account<'info, CashAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The provided amount must be greater than zero.")]
    InvalidAmount, 
    #[msg("Signer does not have access to call this instruction.")]
    InvalidSigner,
}