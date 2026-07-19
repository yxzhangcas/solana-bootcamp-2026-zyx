import { LAMPORTS_PER_SOL } from "@solana/client";
import { useBalance, useWalletConnection } from "@solana/react-hooks";
import { useState } from "react";
import { BACKEND_URL } from "../constant";
import { formatSol } from "../utils";
import { IconDollar, Plus } from "./Icon";

export default function BalanceDisplay() {
  const { wallet } = useWalletConnection();
  const [airdropStatus, setAirdropStatus] = useState('');
  const [airdropping, setAirdropping] = useState(false);

  const walletAddress = wallet?.account.address ? wallet.account.address : undefined;
  const walletBalance = useBalance(walletAddress, { watch: true, fetch: true });
  let displayBalance = walletBalance.lamports ? formatSol(walletBalance.lamports) : null;

  const requestAirdrop = async () => {
    if (!walletAddress || airdropping) return;
    setAirdropping(true);
    setAirdropStatus("Requesting...");
    try {
      const response = await fetch(`${BACKEND_URL}/api/airdrop`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ address: walletAddress, amount: Number(LAMPORTS_PER_SOL) })
      });
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Airdrop failed');
      }
      setAirdropStatus('Success!');
      setTimeout(() => setAirdropStatus(''), 2000);
    } catch (e) {
      console.error(e)
      setAirdropStatus('Failed');
      setTimeout(() => setAirdropStatus(''), 3000);
    } finally {
      setAirdropping(false);
    }
  }

  return (
    <div className="card p-5">
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-4">
          <IconDollar />
          <div>
            <p className="text-[#8d8d94] text-sm">Available Balance</p>
            <p className="text-white text-2xl font-semibold">
              {walletBalance.fetching ? (
                <span className="animate-pulse">Loading...</span>
              ) : displayBalance ? (
                <>
                  {displayBalance}
                  <span className="text-[#5c5e66] text-base font-normal ml-2">SOL</span>
                </>
              ) : ("N/A")}
            </p>
          </div>
        </div>
        <button onClick={requestAirdrop} disabled={airdropping} className="btn btn-outline text-sm">
          {airdropStatus ? (
            <span className={airdropStatus === 'Failed' ? 'text-red-400' : airdropStatus === 'Success!' ? 'text-[#14F195]' : ''}>
              {airdropStatus}
            </span>
          ) : (
            <>
              <Plus />
              Airdrop
            </>
          )}
        </button>
      </div>
    </div>
  )
}