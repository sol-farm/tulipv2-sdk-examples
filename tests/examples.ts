import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Examples }  from '../target/types/examples';
import * as BufferLayout  from 'buffer-layout';
import * as serumAssoToken  from '@project-serum/associated-token';
import * as splToken  from '@solana/spl-token';
import * as assert  from 'assert';

import {
  createAssociatedTokenAccount, deriveMultiDepositStateTransitionAddress,
  deriveOrcaDDCompoundQueueAddress, deriveTrackingOrcaDDQueueAddress, deriveOrcaDDWithdrawQueueAddress,
  deriveManagementAddress, deriveRaydiumUserStakeInfoAddress, deriveSharesMintAddress, deriveVaultAddress, deriveVaultPdaAddress, deriveWithdrawQueueAddress, deriveCompoundQueueAddress, deriveSerumTradeAccount, deriveSerumTradePdaAddress, deriveSerumTradeOpenOrdersAddress, deriveSerumFeeRecipientAddress, deriveTrackingAddress, deriveTrackingPdaAddress, deriveTrackingQueueAddress, findAssociatedStakeInfoAddress, deriveLendingPlatformAccountAddress, deriveLndingPlatformInformationAccountAddress, deriveLendingPlatformConfigDataAddress, deriveMangoAccountAddress, deriveOrcaUserFarmAddress, deriveEphemeralTrackingAddress,
  deriveQuarryVaultConfigDataAddress, deriveSunnyVaultAddress
} from "../ts/utils";
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
describe('examples', () => {
  let provider = anchor.Provider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.Examples as Program<Examples>;

  const programId = program.programId;
  const usdcv1Vault = new anchor.web3.PublicKey("3wPiV9inTGexMZjp6x5Amqwp2sRNtpSheG8Hbv2rgq8W");
  const usdcv1VaultPda = new anchor.web3.PublicKey("14fdy6YXbhDgnVQz4VcgSGgUcZ35eE48SKDrfqF87NUP");
  const usdcv1SharesMint = new anchor.web3.PublicKey("Cvvh8nsKZet59nsDDo3orMa3rZnPWQhpgrMCVcRDRgip");
  const usdcv1DepositQueue = new anchor.web3.PublicKey("36KtHLHxcGnrfEb2GLwPcbN9nHUkeoi3gd6rMQj8wwVj");
  const usdcTokenMint = new anchor.web3.PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
  const associatedTokenProgramId = new anchor.web3.PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
  const v2VaultsProgramId = new anchor.web3.PublicKey("TLPv2tuSVvn3fSk8RgW3yPddkp5oFivzZV3rA9hQxtX");
  
  let depositTrackingAccount: anchor.web3.PublicKey;
  let depositTrackingPda: anchor.web3.PublicKey;
  let depositTrackingQueueAccount: anchor.web3.PublicKey;
  let depositTrackingHoldAccount: anchor.web3.PublicKey;
  
  let yourUnderlyingTokenAccount: anchor.web3.PublicKey;
  let yourSharesTokenAccount: anchor.web3.PublicKey;

  it('registers deposit tracking account', async () => {
    console.log("progrmaId ", programId)
    console.log("usdcv1 vault ", usdcv1Vault)
    console.log("provider", provider.wallet.publicKey)
    let [_depositTrackingAccount, _trackingNonce] = await deriveTrackingAddress(
      v2VaultsProgramId,
      usdcv1Vault,
      provider.wallet.publicKey,
    );
    depositTrackingAccount = _depositTrackingAccount;
    let [_depositTrackingPda, _depositTrackingPdaNonce] = await deriveTrackingPdaAddress(
      v2VaultsProgramId,
      depositTrackingAccount,
    );
    depositTrackingPda = _depositTrackingPda;
    let [_depositTrackingQueueAccount, _queueNonce] = await deriveTrackingQueueAddress(
      v2VaultsProgramId,
      depositTrackingPda,
    );
    depositTrackingQueueAccount = _depositTrackingQueueAccount;
    depositTrackingHoldAccount = await serumAssoToken.getAssociatedTokenAddress(
      depositTrackingPda,
      usdcv1SharesMint,
    );
    const authority = provider.wallet;
    console.log("sending register deposit tracking account tx")
    let tx = await program.methods.registerDepositTrackingAccount().accounts({
      authority: provider.wallet.publicKey,
      vault: usdcv1Vault,
      depositTrackingAccount,
      depositTrackingQueueAccount,
      depositTrackingHoldAccount,
      sharesMint: usdcv1SharesMint,
      underlyingMint: usdcTokenMint,
      depositTrackingPda,
      tokenProgram: splToken.TOKEN_PROGRAM_ID,
      associatedTokenProgram: associatedTokenProgramId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      vaultProgram: v2VaultsProgramId,
    }).signers().rpc();
    console.log("sent register deposit tracking account tx ", tx);
  });
  it('issues shares', async () => {
    yourUnderlyingTokenAccount = await createAssociatedTokenAccount(
      provider,
      provider.wallet.publicKey,
      usdcTokenMint,
    );
    yourSharesTokenAccount = await createAssociatedTokenAccount(
      provider,
      provider.wallet.publicKey,
      usdcv1SharesMint,
    )
    let tx = await program.methods.issueShares(new anchor.BN(0)).accounts({
        authority: provider.wallet.publicKey,
        vault: usdcv1Vault,
        depositTrackingAccount,
        depositTrackingPda,
        vaultPda: usdcv1VaultPda,
        sharesMint: usdcv1SharesMint,
        receivingSharesAccount: depositTrackingHoldAccount,
        depositingUnderlyingAccount: yourUnderlyingTokenAccount,
        vaultUnderlyingAccount: usdcv1DepositQueue,
        systemProgram: anchor.web3.SystemProgram.programId,
        vaultProgram: v2VaultsProgramId,
        tokenProgram: splToken.TOKEN_PROGRAM_ID,
    }).signers().rpc();
  })
  it("withdraws from deposit tracking account", async () => {
    let tx = await program.methods.withdrawDepositTracking(new anchor.BN(0)).accounts({
      authority: provider.wallet.publicKey,
      depositTrackingAccount,
      depositTrackingPda,
      depositTrackingHoldAccount,
      receivingSharesAccount: yourSharesTokenAccount,
      sharesMint: usdcv1SharesMint,
      vault: usdcv1Vault,
      clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      vaultProgram: v2VaultsProgramId,
      tokenProgram: splToken.TOKEN_PROGRAM_ID,
    }).signers().rpc();
  })
});
