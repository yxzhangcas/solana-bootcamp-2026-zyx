import { useEffect, useRef, useState } from "react";
import { ellipsify } from "../lib/explorer";
import { useWallet } from "../providers/wallet-provider";
import {
  AnimateSpin,
  DisconnectIcon,
  DollarCircle,
  OpenCloseArrow,
} from "../utils/icon";

export default function WalletButton() {
  const { connectors, connect, disconnect, wallet, status } = useWallet();
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const address = wallet?.account.address;

  const handleDisconnect = () => {
    disconnect();
    setIsOpen(false);
  };
  const handleConnect = (id: string) => {
    connect(id);
    setIsOpen(false);
  };

  useEffect(() => {
    function handleClickOutside(event: MouseEvent): void {
      const target = event.target as Node;
      if (dropdownRef.current && !dropdownRef.current.contains(target)) {
        setIsOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  if (status === "connected" && address) {
    return (
      <div className="relative" ref={dropdownRef}>
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="flex items-center gap-2 rounded-lg border border-border-low bg-card px-3 py-2 text-sm font-medium transition hover:bg-cream/50"
        >
          <span className="h-2 w-2 rounded-full bg-green-500" />
          <span className="font-mono">{ellipsify(address, 4)}</span>
          <OpenCloseArrow size={4} isOpen={isOpen} />
        </button>
        {isOpen && (
          <div className="absolute right-0 top-full mt-2 w-64 rounded-lg border border-border-low bg-card p-3 shadow-lg z-100">
            <div className="mb-3 pb-3 border-b border-border-low">
              <p className="text-xs text-muted mb-1">Connected wallet</p>
              <p className="font-mono text-xs break-all">{address}</p>
            </div>
            <div className="space-y-2">
              <a
                href="https://faucet.solana.com/"
                className="flex items-center gap-2 w-full rounded-md px-3 py-2 text-sm text-left transition hover:bg-cream/50"
              >
                <DollarCircle size={4} />
                Get devnet SOL
              </a>
              <button
                onClick={handleDisconnect}
                className="flex items-center gap-2 w-full rounded-md px-3 py-2 text-sm text-left text-red-600 transition hover:bg-red-50"
              >
                <DisconnectIcon size={4} />
                Disconnect
              </button>
            </div>
          </div>
        )}
      </div>
    );
  }
  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        disabled={status === "connecting"}
        className="flex items-center gap-2 rounded-lg bg-foreground px-4  py-2 text-sm font-medium text-background transition hover:opacity-90 disabled:opacity-60"
      >
        {status === "connecting" ? (
          <>
            <AnimateSpin size={4} />
            Connecting...
          </>
        ) : (
          <>
            Connect Wallet
            <OpenCloseArrow size={4} isOpen={isOpen} />
          </>
        )}
      </button>
      {isOpen && status !== "connecting" && (
        <div className="absolute right-0 top-full mt-2 w-64 rounded-lg border border-border-low bg-card p-2 shadow-lg z-100">
          <p className="px-3 py-2 text-xs text-muted">Select a wallet</p>
          {connectors.map((connector) => (
            <button
              key={connector.id}
              onClick={() => handleConnect(connector.id)}
              className="flex items-center gap-3 w-full rounded-md px-3 py-2.5 text-sm text-left transition hover:bg-cream/50"
            >
              <span className="h-2 w-2 rounded-full bg-border-low" />
              {connector.name}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
