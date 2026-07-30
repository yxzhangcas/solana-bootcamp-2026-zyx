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

  console.log("🚀 Initializing Labubu Collection...");
  console.log("Program ID:", program.programId.toString());
  console.log("Collection PDA:", collectionPda.toString());

  try {
    const tx = await program.methods
      .initializeCollection()
      .accounts({
        authority: provider.wallet.publicKey,
      })
      .rpc();

    console.log("✅ Collection initialized successfully!");
    console.log("Transaction signature:", tx);

    // Read collection account to verify
    const collectionAccount = await program.account.labubuCollection.fetch(
      collectionPda
    );

    console.log("Total supply:", collectionAccount.remainingSupply);
    console.log("Total minted:", collectionAccount.totalMinted);
  } catch (error) {
    console.error("❌ Initialization failed:", error);
    throw error;
  }
}

main();

// yxzhang@SolanaLearn:labubu-vault$ export ANCHOR_WALLET=~/.config/solana/id.json
// yxzhang@SolanaLearn:labubu-vault$ ts-node scripts/initialize.ts 