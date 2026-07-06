import { address, lamports } from '@solana/kit';
import express, { type Express, type Request, type Response } from 'express';
import { LAMPORTS_PER_SOL, PORT, RPC } from './constant';
import { deposit } from './deposit';
import { withdraw } from './withdraw';

const app: Express = express();
app.use(express.json());  // 用于解析JSON输入

async function airdrop(req: Request, res: Response) {
  try {
    // 解析参数: address, amount
    const { address: addressStr, amount } = req.body;
    if (!addressStr) {
      return res.status(400).json({ error: "Missing address" });
    }
    const recipient = address(addressStr);
    const lamportsAmount = lamports(BigInt(amount || LAMPORTS_PER_SOL));
    console.log(`Requesting airdrop of ${Number(lamportsAmount) / Number(LAMPORTS_PER_SOL)} SOL to ${addressStr}...`);

    // 请求空投，并检查空投结果
    const airdropSignature = await RPC.requestAirdrop(recipient, lamportsAmount).send();
    for (let i = 0; i < 30; i++) {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      const { value: [status] } = await RPC.getSignatureStatuses([airdropSignature]).send();
      if (status?.confirmationStatus === "confirmed" || status?.confirmationStatus === "finalized") {
        console.log(`Airdrop successful: ${airdropSignature}`);
        res.json({ success: true, signature: airdropSignature });
        return;
      }
    }
    // 超时抛出错误
    throw new Error("Airdrop confirmation timeout");
  } catch (error) {
    console.error("Airdrop error:", error);
    res.status(500).json({ error: error instanceof Error ? error.message : "Unknown error" });
  }

}
async function health(_req: Request, res: Response) {
  res.json({ status: "OK" });
}

app.post("/api/deposit", deposit);
app.post("/api/withdraw", withdraw);
app.post("/api/airdrop", airdrop);
app.get("/api/health", health);

app.listen(PORT, () => {
  console.log(`Backend API server running on http://localhost:${PORT}`);
  console.log("Endpoints:");
  console.log("  POST /api/deposit - Generate deposit note");
  console.log("  POST /api/withdraw - Generate withdrawal proof");
  console.log("  POST /api/airdrop - Request devnet SOL airdrop");
  console.log("  GET /api/health - Health check");
});