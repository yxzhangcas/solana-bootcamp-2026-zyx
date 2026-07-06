import { getProgramDerivedAddress } from "@solana/kit";
import { poseidon2Hash } from "@zkpassport/poseidon2";
import * as crypto from "crypto";
import { type Request, type Response } from 'express';
import { BN254_MODULUS, EMPTY_TREE_ZEROS, MIN_DEPOSIT, PROGRAM_ID, RPC, TREE_DEPTH } from './constant';

async function getNextLeafIndex(): Promise<number> {
  try {
    const pool = (await getProgramDerivedAddress({ programAddress: PROGRAM_ID, seeds: [Buffer.from("pool")] }))[0];
    const poolAccount = await RPC.getAccountInfo(pool, { encoding: "base64" }).send();
    if (!poolAccount.value) {
      console.log("Pool not initialized, using index 0");
      return 0;
    }
    const data = Buffer.from(poolAccount.value.data[0], "base64");
    const nextLeafIndex = data.readBigUint64LE(40);
    return Number(nextLeafIndex);
  } catch (error) {
    console.log("Error fetching leaf index, using 0:", error);
    return 0;
  }
}

function generateRandomField(): bigint {
  let value: bigint;
  do {
    const bytes = crypto.randomBytes(32);
    value = BigInt("0x" + bytes.toString("hex"));
  } while (value >= BN254_MODULUS);
  return value;
}

function computeHashes(nullifier: bigint, secret: bigint, amount: bigint): {
  commitment: string, nullifierHash: string
} {
  const commitment: bigint = poseidon2Hash([nullifier, secret, amount]);
  const commitmentHex = "0x" + commitment.toString(16).padStart(64, "0");
  const nullifierHash: bigint = poseidon2Hash([nullifier]);
  const nullifierHashHex = "0x" + nullifierHash.toString(16).padStart(64, "0");
  return {
    commitment: commitmentHex,
    nullifierHash: nullifierHashHex,
  }
}

function computeMerkleRoot(commitment: string, leafIndex: number): string {
  const leaf = BigInt(commitment);
  let current = leaf;
  let idx = leafIndex;
  for (let i = 0; i < TREE_DEPTH; i++) {
    const sibling = BigInt(EMPTY_TREE_ZEROS[i]);
    const isRight = (idx & 1) === 1;
    if (isRight) {
      current = poseidon2Hash([sibling, current]);
    } else {
      current = poseidon2Hash([current, sibling]);
    }
    idx = idx >> 1;
  }
  return "0x" + current.toString(16).padStart(64, "0");
}

// 并未执行合约方法修改状态，仅查询了账户信息，生成相关的ZK信息
export async function deposit(req: Request, res: Response) {
  try {
    // check input amount
    const { amount } = req.body;
    if (!amount || isNaN(Number(amount))) {
      return res.status(400).json({ error: "Invalid amount: must be a number" });
    }
    const amountNum = Number(amount);
    if (amountNum <= 0) {
      return res.status(400).json({ error: "Invalid amount: must be positive" });
    }
    if (!Number.isInteger(amountNum)) {
      return res.status(400).json({ error: "Invalid amount: must be an integer (lamports)" });
    }
    if (amountNum < MIN_DEPOSIT) {
      return res.status(400).json({
        error: `Invalid amount: minimum deposit is ${MIN_DEPOSIT} lamports (0.001 SOL)`,
      });
    }

    const leafIndex = await getNextLeafIndex();
    console.log(
      `Generating deposit for ${amount} lamports at leaf index ${leafIndex}...`
    );

    // 生成随机的符合要求的nullifier secret
    const nullifier = generateRandomField();
    const secret = generateRandomField();
    const amountBigInt = BigInt(amount);
    // Compute hashes using JavaScript Poseidon2 (no nargo needed!)
    const hashes = computeHashes(nullifier, secret, amountBigInt);
    const merkleRoot = computeMerkleRoot(hashes.commitment, leafIndex);
    const commitmentBytes = Array.from(Buffer.from(hashes.commitment.slice(2), "hex"));
    const merkleRootBytes = Array.from(Buffer.from(merkleRoot.slice(2), "hex"));

    const depositNote = {
      nullifier: nullifier.toString(),
      secret: secret.toString(),
      amount: amount.toString(),
      commitment: hashes.commitment,
      nullifierHash: hashes.nullifierHash,
      merkleRoot: merkleRoot,
      leafIndex: leafIndex,
      timestamp: Date.now(),
    };
    console.log(`Deposit note generated: ${hashes.commitment.slice(0, 16)}... at index ${leafIndex}`);

    res.json({
      depositNote,
      onChainData: {
        commitment: commitmentBytes,
        newRoot: merkleRootBytes,
        amount: amount.toString(),
      },
    });
  } catch (error) {
    console.error("Deposit generation error:", error);
    res.status(500).json({ error: error instanceof Error ? error.message : "Unknown error" });
  }
}