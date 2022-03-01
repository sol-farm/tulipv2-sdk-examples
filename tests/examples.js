"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
const anchor = __importStar(require("@project-serum/anchor"));
const splToken = __importStar(require("@solana/spl-token"));
const utils_1 = require("../ts/utils");
describe('examples', () => {
    let provider = anchor.Provider.env();
    // Configure the client to use the local cluster.
    anchor.setProvider(provider);
    const program = anchor.workspace.Examples;
    const programId = program.programId;
    const usdcv1Vault = new anchor.web3.PublicKey("3wPiV9inTGexMZjp6x5Amqwp2sRNtpSheG8Hbv2rgq8W");
    const usdcv1SharesMint = new anchor.web3.PublicKey("Cvvh8nsKZet59nsDDo3orMa3rZnPWQhpgrMCVcRDRgip");
    const usdcTokenMint = new anchor.web3.PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    const associatedTokenProgramId = new anchor.web3.PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
    const v2VaultsProgramId = new anchor.web3.PublicKey("TLPv2tuSVvn3fSk8RgW3yPddkp5oFivzZV3rA9hQxtX");
    let depositTrackingAcccount;
    let depositTrackingPda;
    let depositTrackingQueueAccount;
    let depositTrackingHoldAccount;
    it('Is initialized!', () => __awaiter(void 0, void 0, void 0, function* () {
        console.log("progrmaId ", programId);
        console.log("usdcv1 vault ", usdcv1Vault);
        console.log("provider", provider.wallet.publicKey);
        let [_depositTrackingAccount, _trackingNonce] = yield (0, utils_1.deriveTrackingAddress)(programId, usdcv1Vault, provider.wallet.publicKey);
        depositTrackingAcccount = _depositTrackingAccount;
        let [_depositTrackingPda, _depositTrackingPdaNonce] = yield (0, utils_1.deriveTrackingPdaAddress)(programId, depositTrackingAcccount);
        depositTrackingPda = _depositTrackingPda;
        let [_depositTrackingQueueAccount, _queueNonce] = yield (0, utils_1.deriveTrackingQueueAddress)(programId, depositTrackingPda);
        depositTrackingQueueAccount = _depositTrackingQueueAccount;
        depositTrackingHoldAccount = yield (0, utils_1.createAssociatedTokenAccount)(provider, depositTrackingPda, usdcv1SharesMint);
        const authority = provider.wallet;
        console.log("sending register deposit tracking account tx");
        let tx = program.methods.registerDepositTrackingAccount().accounts({
            authority: provider.wallet.publicKey,
            vault: usdcv1Vault,
            depositTrackingAcccount,
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
        }).signers([provider.wallet]).rpc();
        //const tx = await program.rpc.registerDepositTrackingAccount({
        //  authority: provider.wallet.publicKey,
        //  vault: usdcv1Vault,
        //  depositTrackingAcccount,
        //  depositTrackingQueueAccount,
        //  depositTrackingHoldAccount,
        //  sharesMint: usdcv1SharesMint,
        //  underlyingMint: usdcTokenMint,
        //  depositTrackingPda,
        //  tokenProgram: splToken.TOKEN_PROGRAM_ID,
        //  associatedTokenProgram: associatedTokenProgramId,
        //  rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        //  systemProgram: anchor.web3.SystemProgram.programId,
        //  vaultProgram: v2VaultsProgramId,
        //});
        console.log("sent register deposit tracking accoutn tx ", tx);
    }));
});
