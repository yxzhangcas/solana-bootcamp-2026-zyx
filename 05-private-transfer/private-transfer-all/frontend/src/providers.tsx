import { autoDiscover, createClient } from "@solana/client";
import { SolanaProvider } from "@solana/react-hooks";
import { PropsWithChildren } from "react";
import { ENDPOINT_URL, ENDPOINT_WS_URL } from "./constant";

const client = createClient({
  endpoint: ENDPOINT_URL,
  websocket: ENDPOINT_WS_URL, // 需要配置ws_url才能支持自动获取balance和自动刷新
  walletConnectors: autoDiscover(),
});

export function Providers({ children }: PropsWithChildren) {
  return <SolanaProvider client={client}>{children}</SolanaProvider>;
}
