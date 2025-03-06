import { Connection, Keypair } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { SalaryService } from "./salary";

async function main() {
    // 初始化连接和程序
    const connection = new Connection("https://api.devnet.solana.com");
    const provider = new anchor.AnchorProvider(
        connection,
        new anchor.Wallet(Keypair.generate()),
        { commitment: "confirmed" }
    );
    
    // 加载你的程序 IDL
    const program = new anchor.Program(IDL, PROGRAM_ID, provider);
    
    const salaryService = new SalaryService(program, connection);

    // 存款示例
    const sender = Keypair.generate(); // 实际使用时应该是连接的钱包
    const receiver = Keypair.generate().publicKey;
    const amount = 1000000; // 1 USDT (6位小数)
    const unlockTime = Math.floor(Date.now() / 1000) + 86400; // 24小时后

    await salaryService.deposit(sender, receiver, amount, unlockTime);

    // 提现示例
    const receiverWallet = Keypair.generate(); // 实际使用时应该是接收方的钱包
    await salaryService.withdraw(receiverWallet);
}

main().catch(console.error); 