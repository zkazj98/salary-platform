import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { 
    PublicKey, 
    SystemProgram, 
    SYSVAR_RENT_PUBKEY,
    Connection,
    Keypair 
} from "@solana/web3.js";
import {
    TOKEN_PROGRAM_ID,
    getAssociatedTokenAddress,
    createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { SalaryPlatform } from "./types/salary_platform"; // 你的 IDL 类型

export class SalaryService {
    private program: Program<SalaryPlatform>;
    private connection: Connection;

    constructor(program: Program<SalaryPlatform>, connection: Connection) {
        this.program = program;
        this.connection = connection;
    }

    async deposit(
        sender: anchor.web3.Keypair,
        receiver: PublicKey,
        amount: number,
        unlockTime: number,
    ) {
        const USDT_MINT = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
        
        // 获取发送者的代币账户
        const senderTokenAccount = await getAssociatedTokenAddress(
            USDT_MINT,
            sender.publicKey
        );

        // 获取 escrow account PDA
        const [escrowAccount] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("escrow"),
                receiver.toBuffer(),
            ],
            this.program.programId
        );

        // 获取 escrow token account PDA
        const [escrowTokenAccount] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("escrow"),
                escrowAccount.toBuffer(),
            ],
            this.program.programId
        );

        try {
            const tx = await this.program.methods
                .deposit(
                    new anchor.BN(amount),
                    new anchor.BN(unlockTime),
                )
                .accounts({
                    sender: sender.publicKey,
                    receiver: receiver,
                    sendTokenAccount: senderTokenAccount,
                    escrowAccount: escrowAccount,
                    escrowTokenAccount: escrowTokenAccount,
                    usdcMint: USDT_MINT,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    systemProgram: SystemProgram.programId,
                })
                .signers([sender])
                .rpc();

            console.log("Deposit transaction signature", tx);
            return tx;
        } catch (error) {
            console.error("Error in deposit:", error);
            throw error;
        }
    }

    async withdraw(
        receiver: anchor.web3.Keypair,
    ) {
        // 获取 escrow account PDA
        const [escrowAccount] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("escrow"),
                receiver.publicKey.toBuffer(),
            ],
            this.program.programId
        );

        // 获取 escrow token account PDA
        const [escrowTokenAccount] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("escrow"),
                escrowAccount.toBuffer(),
            ],
            this.program.programId
        );

        // 获取接收者的代币账户
        const receiverTokenAccount = await getAssociatedTokenAddress(
            new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            receiver.publicKey
        );

        try {
            const tx = await this.program.methods
                .withdraw()
                .accounts({
                    receiver: receiver.publicKey,
                    escrowAccount: escrowAccount,
                    escrowTokenAccount: escrowTokenAccount,
                    receiverTokenAccount: receiverTokenAccount,
                    tokenProgram: TOKEN_PROGRAM_ID,
                })
                .signers([receiver])
                .rpc();

            console.log("Withdraw transaction signature", tx);
            return tx;
        } catch (error) {
            console.error("Error in withdraw:", error);
            throw error;
        }
    }
} 