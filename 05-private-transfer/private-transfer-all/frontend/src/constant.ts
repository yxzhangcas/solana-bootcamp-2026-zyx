import { address } from "@solana/kit";

export const ENDPOINT_URL = "http://127.0.0.1:8899";
export const ENDPOINT_WS_URL = "ws://127.0.0.1:8900";

export const BACKEND_URL = 'http://localhost:4001';

export const DEFAULT_DEPOSIT_AMOUNT = '0.1';

export const SEEDS = {
  POOL: new Uint8Array([112, 111, 111, 108]), // "pool"
  VAULT: new Uint8Array([118, 97, 117, 108, 116]), // "vault"
  NULLIFIERS: new Uint8Array([110, 117, 108, 108, 105, 102, 105, 101, 114, 115]), // "nullifiers"
} as const;

export const SYSTEM_PROGRAM_ID = address('11111111111111111111111111111111');
export const SUNSPOT_VERIFIER_ID = address('HELhybP3HoggrnWfAxH8Bzc2V98g9sLJg72Xnpv8dyu9')