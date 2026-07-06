import { address, createSolanaRpc } from "@solana/kit";
import path from "path";

export const LAMPORTS_PER_SOL = 1_000_000_000;
export const MIN_DEPOSIT = 1_000_000;
export const PORT = process.env.PORT || 4001;

export const RPC = createSolanaRpc("http://127.0.0.1:8899")
export const PROGRAM_ID = address("3CHgWxcuX4zx4kZqQiQodd3db7qut5jcpTyfq8uUu7um");
export const VERIFIER_DIR = path.resolve(__dirname, "../../verifier");

export const BN254_MODULUS = BigInt(
  "21888242871839275222246405745257275088548364400416034343698204186575808495617"
);
export const TREE_DEPTH = 10;
// Pre-computed zeros for empty Merkle tree (Poseidon2 hashes using @zkpassport/poseidon2)
export const EMPTY_TREE_ZEROS = [
  "0x0000000000000000000000000000000000000000000000000000000000000000",
  "0x0b63a53787021a4a962a452c2921b3663aff1ffd8d5510540f8e659e782956f1",
  "0x0e34ac2c09f45a503d2908bcb12f1cbae5fa4065759c88d501c097506a8b2290",
  "0x21f9172d72fdcdafc312eee05cf5092980dda821da5b760a9fb8dbdf607c8a20",
  "0x2373ea368857ec7af97e7b470d705848e2bf93ed7bef142a490f2119bcf82d8e",
  "0x120157cfaaa49ce3da30f8b47879114977c24b266d58b0ac18b325d878aafddf",
  "0x01c28fe1059ae0237b72334700697bdf465e03df03986fe05200cadeda66bd76",
  "0x2d78ed82f93b61ba718b17c2dfe5b52375b4d37cbbed6f1fc98b47614b0cf21b",
  "0x067243231eddf4222f3911defbba7705aff06ed45960b27f6f91319196ef97e1",
  "0x1849b85f3c693693e732dfc4577217acc18295193bede09ce8b97ad910310972",
];
