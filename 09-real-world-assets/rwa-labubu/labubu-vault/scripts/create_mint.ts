import { AnchorProvider, Program, setProvider, Wallet } from "@anchor-lang/core";
import { Connection, PublicKey } from "@solana/web3.js";
import { Buffer } from "buffer";
import idl from "../target/idl/labubu_vault.json";
import type { LabubuVault } from "../target/types/labubu_vault";

async function main() {
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");
  const wallet = Wallet.local();
  const provider = new AnchorProvider(connection, wallet, { commitment: "confirmed" });
  setProvider(provider);
  const program = new Program<LabubuVault>(idl as LabubuVault, provider);

  const [collectionPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("collection")],
    program.programId
  );

  console.log("🎨 Creating 11 Labubu Mints...");

  const labubuNames = [
    "Zone Out",
    "Ab Roller",
    "Confident",
    "Show Off",
    "Stretch Out",
    "Sweating",
    "Sleeping",
    "Little Bird",
    "Americano",
    "Lay Down",
    "Secret Edition ⭐",
  ];

  for (let i = 1; i <= 11; i++) {
    const [mintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("labubu_mint"), Buffer.from([i])],
      program.programId
    );

    console.log(`  Creating Labubu #${i} (${labubuNames[i - 1]})...`);

    try {
      const tx = await program.methods
        .createMint(i)
        .accounts({
          authority: provider.wallet.publicKey,
          labubuMint: mintPda,
        })
        .rpc();

      console.log(`  ✅ Mint #${i} created successfully: ${mintPda.toString()}`);
    } catch (error) {
      console.error(`  ❌ Mint #${i} creation failed:`, error);
      throw error;
    }
  }

  console.log("🎉 All Mints created successfully!");
}

main();

// yxzhang@SolanaLearn:labubu-vault$ ts-node scripts/create_mint.ts 