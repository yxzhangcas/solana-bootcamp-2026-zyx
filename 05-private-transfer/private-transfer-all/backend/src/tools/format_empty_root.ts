import { poseidon2Hash } from "@zkpassport/poseidon2";
import { TREE_DEPTH } from "../constant";

// Compute the final empty tree root
let current = 0n;
for (let i = 0; i < TREE_DEPTH; i++) {
  current = poseidon2Hash([current, current]);
}

const hex = current.toString(16).padStart(64, "0");
console.log("Empty root (hex):", "0x" + hex);
console.log("");
console.log("Rust format:");
console.log("pub const EMPTY_ROOT: [u8; 32] = [");

// Convert to bytes and format for Rust
const bytes: string[] = [];
for (let i = 0; i < 64; i += 2) {
  bytes.push("0x" + hex.slice(i, i + 2));
}

// Format in rows of 16 bytes
for (let i = 0; i < 32; i += 16) {
  const row = bytes.slice(i, i + 16).join(", ");
  console.log(`    ${row},`);
}
console.log("];");

// Empty root (hex): 0x2a775ea761d20435b31fa2c33ff07663e24542ffb9e7b293dfce3042eb104686

// Rust format:
// pub const EMPTY_ROOT: [u8; 32] = [
//     0x2a, 0x77, 0x5e, 0xa7, 0x61, 0xd2, 0x04, 0x35, 0xb3, 0x1f, 0xa2, 0xc3, 0x3f, 0xf0, 0x76, 0x63,
//     0xe2, 0x45, 0x42, 0xff, 0xb9, 0xe7, 0xb2, 0x93, 0xdf, 0xce, 0x30, 0x42, 0xeb, 0x10, 0x46, 0x86,
// ];