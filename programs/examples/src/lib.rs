use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use vaults::instructions::{new_register_deposit_tracking_account_ix, new_issue_shares_ix};


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod examples {
    use common::msg_panic;

    use super::*;
    pub fn register_deposit_tracking_account(
        ctx: Context<RegisterDepositTrackingAccount>,
    ) -> Result<()> {
        let farm_key = {
            let loader: AccountLoader<vaults::accounts::multi_optimizer::MultiDepositOptimizerV1> = AccountLoader::try_from_unchecked(
                &ctx.accounts.vault_program.key(),
                &ctx.accounts.vault,
            )?;
            {
                let vault = loader.load()?;
                vault.base.farm
            }
        };
        let got_tracking = vaults::accounts::derive_tracking_address(
            ctx.accounts.vault.key,
            ctx.accounts.authority.key,
            ctx.accounts.vault_program.key,
        ).0;
        if ctx.accounts.deposit_tracking_account.key().ne(&got_tracking) {
            msg_panic!("invalid deposit tracking account. got {}, want {}", got_tracking, ctx.accounts.deposit_tracking_account.key());
        }
        let got_pda = vaults::accounts::derive_tracking_pda_address(
            &got_tracking,
            ctx.accounts.vault_program.key,
        ).0;
        if ctx.accounts.deposit_tracking_pda.key().ne(&got_pda) {
            msg_panic!("invalid deposit tracking pda. got {}, want {}", got_pda, ctx.accounts.deposit_tracking_pda.key());
        }
        // create the associate
        {
            let ix = spl_associated_token_account::create_associated_token_account(
                ctx.accounts.authority.key,
                ctx.accounts.deposit_tracking_pda.key,
                &ctx.accounts.shares_mint.key(),
            );
            anchor_lang::solana_program::program::invoke(
                &ix,
                &[
                    ctx.accounts.authority.clone(),
                    ctx.accounts.deposit_tracking_pda.clone(),
                    ctx.accounts.shares_mint.to_account_info(),
                    ctx.accounts.token_program.clone(),
                    ctx.accounts.deposit_tracking_hold_account.clone(),
                    ctx.accounts.rent.to_account_info(),
                ],
            )?;
        }
        {
            let ix = new_register_deposit_tracking_account_ix(
            ctx.accounts.authority.key(),
            ctx.accounts.vault.key(),
            ctx.accounts.deposit_tracking_account.key(),
            ctx.accounts.deposit_tracking_queue_account.key(),
            ctx.accounts.deposit_tracking_hold_account.key(),
            ctx.accounts.shares_mint.key(),
            ctx.accounts.deposit_tracking_pda.key(),
            farm_key.into()
            );
            anchor_lang::solana_program::program::invoke(
                &ix,
                &[
                    ctx.accounts.authority.clone(),
                    ctx.accounts.vault.clone(),
                    ctx.accounts.deposit_tracking_account.clone(),
                    ctx.accounts.deposit_tracking_queue_account.clone(),
                    ctx.accounts.deposit_tracking_hold_account.clone(),
                    ctx.accounts.shares_mint.to_account_info(),
                    ctx.accounts.deposit_tracking_pda.clone(),
                    ctx.accounts.rent.to_account_info(),
                ],
            )?;
        }
        Ok(())
    }
    pub fn issue_shares(ctx: Context<IssueShares>, amount: u64) -> Result<()> {
        let farm_key = {
            let loader: AccountLoader<vaults::accounts::multi_optimizer::MultiDepositOptimizerV1> = AccountLoader::try_from_unchecked(
                &ctx.accounts.vault_program.key(),
                &ctx.accounts.vault,
            )?;
            {
                let vault = loader.load()?;
                vault.base.farm
            }
        };
        /*
            if this error is returned, it means the depositing_underlying_account
            has less tokens (X) then requested deposit amount (Y)
            Program log: RUNTIME ERROR: a(X) < b(Y)
            Program log: panicked at 'RUNTIME ERROR: a(0) < b(1)', programs/vaults/src/vault_instructions/deposit_tracking/acl_helpers.rs:198:9
        */
        let ix = new_issue_shares_ix(
            ctx.accounts.authority.key(),
            ctx.accounts.vault.key(),
            ctx.accounts.deposit_tracking_account.key(),
            ctx.accounts.deposit_tracking_pda.key(),
            ctx.accounts.vault_pda.key(),
            ctx.accounts.vault_underlying_account.key(),
            ctx.accounts.shares_mint.key(),
            ctx.accounts.receiving_shares_account.key(),
            ctx.accounts.depositing_underlying_account.key(),
            farm_key.into(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.clone(),
                ctx.accounts.vault.clone(),
                ctx.accounts.deposit_tracking_account.clone(),
                ctx.accounts.deposit_tracking_pda.clone(),
                ctx.accounts.vault_pda.clone(),
                ctx.accounts.vault_underlying_account.to_account_info(),
                ctx.accounts.shares_mint.to_account_info(),
                ctx.accounts.receiving_shares_account.to_account_info(),
                ctx.accounts.depositing_underlying_account.to_account_info(),
            ],
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct RegisterDepositTrackingAccount<'info> {
    #[account(mut, signer)]
    pub authority: AccountInfo<'info>,
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_account: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_queue_account: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_hold_account: AccountInfo<'info>,
    #[account(mut)]
    pub shares_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub underlying_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub deposit_tracking_pda: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub vault_program: AccountInfo<'info>,
}


#[derive(Accounts)]
pub struct IssueShares<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_account: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_pda: AccountInfo<'info>,
    pub vault_pda: AccountInfo<'info>,
    #[account(mut)]
    pub shares_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    /// the account which will receive the issued shares
    /// this is the deposit_tracking_hold_account
    pub receiving_shares_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// the account owned by the authority which contains the underlying tokens
    /// we want to deposit in exchange for the vault shares
    pub depositing_underlying_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// the underlying token account that is owned by the vault pda
    /// which holds the underlying tokens until they are swept into the farm.
    ///
    /// also known as the deposit queue account
    pub vault_underlying_account: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub vault_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}