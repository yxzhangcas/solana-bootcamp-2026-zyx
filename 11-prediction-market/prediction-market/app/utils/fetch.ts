import { Address } from "@solana/kit";
import { VAULT_PROGRAM_ADDRESS } from "../generated/vault";
import { CLUSTER_URLS, ClusterMoniker } from "../lib/solana-client";

const MARKET_DISCRIMINATOR_BASE58 = "dkokXHR3DTw";
const USER_POSITION_DISCRIMINATOR_BASE58 = "j9SjDYAWesU";

export async function rpcFetchMarkets(cluster: ClusterMoniker) {
  const response = await fetch(CLUSTER_URLS[cluster], {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getProgramAccounts",
      params: [
        VAULT_PROGRAM_ADDRESS,
        {
          encoding: "base64",
          commitment: "confirmed",
          filters: [
            {
              memcmp: {
                offset: 0,
                bytes: MARKET_DISCRIMINATOR_BASE58,
              },
            },
          ],
        },
      ],
    }),
  });
  const result = await response.json();
  if (result.error) throw new Error(result.error.message);
  return result;
}

export async function rpcFetchPositions(
  cluster: ClusterMoniker,
  walletAddress: Address
) {
  const response = await fetch(CLUSTER_URLS[cluster], {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getProgramAccounts",
      params: [
        VAULT_PROGRAM_ADDRESS,
        {
          encoding: "base64",
          commitment: "confirmed",
          filters: [
            {
              memcmp: {
                offset: 0,
                bytes: USER_POSITION_DISCRIMINATOR_BASE58,
              },
            },
            {
              memcmp: {
                offset: 40, // 8 (discriminator) + 32 (market) = user field
                bytes: walletAddress,
              },
            },
          ],
        },
      ],
    }),
  });
  const result = await response.json();
  if (result.error) throw new Error(result.error.message);
  return result;
}

export async function rpcFetchMarketAccounts(
  cluster: ClusterMoniker,
  marketAddresses: Address[]
) {
  const response = await fetch(CLUSTER_URLS[cluster], {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 2,
      method: "getMultipleAccounts",
      params: [
        marketAddresses,
        { encoding: "base64", commitment: "confirmed" },
      ],
    }),
  });
  const result = await response.json();
  return result;
}

export async function rpcFetchPositionAccount(
  cluster: ClusterMoniker,
  positionAddress: Address
) {
  const response = await fetch(CLUSTER_URLS[cluster], {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getAccountInfo",
      params: [
        positionAddress,
        { encoding: "base64", commitment: "confirmed" },
      ],
    }),
  });
  const result = await response.json();
  if (!result.result?.value) return null;
  return result;
}
