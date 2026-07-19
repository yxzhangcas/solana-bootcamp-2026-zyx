import { useWalletConnection } from "@solana/react-hooks";
import { useState } from "react";
import { truncateAddress } from "../utils";
import { IconCashSmall, AnimateSpin, GreenPoint } from "./Icon";
import "./wallet-connection.css";

export default function WalletConnection() {
  const { connectors, connect, disconnect, connecting, wallet } = useWalletConnection();
  const [showDropdown, setShowDropdown] = useState(false);

  const walletAddress = wallet?.account.address ? wallet.account.address : undefined;

  if (walletAddress) {
    return (
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-[#111214] border border-[#232529]">
          <GreenPoint />
          <span className="text-[#8b8d94] text-sm font-mono">{truncateAddress(walletAddress, 6)}</span>
        </div>
        <button onClick={disconnect} className="text-sm btn btn-ghost">Disconnect</button>
      </div>
    )
  }

  return (
    <div className="relative">
      <button onClick={() => setShowDropdown(!showDropdown)} disabled={connecting} className="btn btn-primary">
        {connecting ? (
          <span className="flex items-center gap-2"><AnimateSpin />Connecting...</span>
        ) : ('Connect Wallet')}
      </button>
      {showDropdown && connectors.length > 0 ? (
        <>
          {/* 对任意未预先定义点击事件的位置进行点击，收起下拉菜单 */}
          <div className="fixed inset-0 z-10" onClick={() => setShowDropdown(false)} />
          <div className="absolute right-0 mt-2 w-52 bg-[#111214] border border-[#232529] rounded-xl shadow-2xl z-20 overflow-hidden animate-fade-in">
            <div className="p-2">
              {connectors.map((connector) => (
                <button
                  key={connector.id}
                  onClick={() => { connect(connector.id); setShowDropdown(false) }}
                  className="flex items-center gap-3 w-full text-left px-3 py-1 text-[#f4f4f5] hover:bg-[#18191c] rounded-lg transition-colors"
                >
                  <IconCashSmall />
                  <span className="text-sm font-medium">{connector.name}</span>
                </button>
              ))}
            </div>
          </div>
        </>
      ) : (<></>)}
    </div>
  )
}