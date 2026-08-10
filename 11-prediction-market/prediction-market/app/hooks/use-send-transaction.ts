"use client";

import type { Instruction } from "@solana/kit";
import { createClient } from "@solana/kit-client-rpc";
import { useCallback, useMemo, useState } from "react";
import { useSWRConfig } from "swr";
import { getClusterUrl, getClusterWsConfig } from "../lib/solana-client";
import { useCluster } from "../providers/cluster-provider";
import { useWallet } from "../providers/wallet-provider";

export function useSendTransaction() {
  const { signer } = useWallet();
  const { cluster } = useCluster();
  const { mutate } = useSWRConfig();
  const [isSending, setIsSending] = useState(false);

  // 创建了新的client，并未使用provider中已经创建的client(先于wallet创建，未配置signer，仅支持查询和airdrop)
  const txClient = useMemo(
    () =>
      signer
        ? createClient({
            url: getClusterUrl(cluster),
            rpcSubscriptionsConfig: getClusterWsConfig(cluster),
            payer: signer,
          })
        : null,
    [cluster, signer]
  );

  const send = useCallback(
    async ({ instructions }: { instructions: readonly Instruction[] }) => {
      if (!txClient) throw new Error("Wallet not connected");

      setIsSending(true);
      try {
        const result = await txClient.sendTransaction([...instructions]);
        mutate((key: unknown) => Array.isArray(key) && key[0] === "balance");
        return result.context.signature;
      } finally {
        setIsSending(false);
      }
    },
    [txClient, mutate]
  );

  return { send, isSending };
}
