import { LAMPORTS_PER_SOL } from "@solana/client";
import { getAddressEncoder, getBytesEncoder, getProgramDerivedAddress } from "@solana/kit";
import { useSendTransaction, useWalletConnection } from "@solana/react-hooks";
import { useState } from "react";
import { BACKEND_URL, DEFAULT_DEPOSIT_AMOUNT, SEEDS, SYSTEM_PROGRAM_ID } from "../constant";
import { getDepositInstructionDataEncoder, PRIVATE_TRANSFER_PROGRAM_ADDRESS } from "../generated/src/generated";
import { DepositApiResponse, DepositNote } from "../types";
import { AnimateSpin, CheckIcon, CopyIcon, IconDeposit, IconWarn } from "./Icon";

interface DepositSectionProps {
  handleDepositComplete: (note: DepositNote) => void
  depositNote: DepositNote | null
  handleClearNote: () => void
}

export default function Deposit({ handleDepositComplete, depositNote, handleClearNote }: DepositSectionProps) {
  const { wallet } = useWalletConnection();
  const { send: sendTransaction, isSending } = useSendTransaction();

  const [amount, setAmount] = useState(DEFAULT_DEPOSIT_AMOUNT);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState('');
  const [copied, setCopied] = useState(false);

  const walletAddress = wallet?.account.address;
  const isProcessing = loading || isSending;

  const handleCopy = async () => {
    if (!depositNote) return;
    try {
      await navigator.clipboard.writeText(JSON.stringify(depositNote));
      setCopied(true);
      setTimeout(() => setCopied(false), 2_000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }
  const handleDeposit = async () => {
    if (!walletAddress) return;
    setLoading(true);
    setStatus('Generating Deposit note...');
    try {
      const lamports = Math.floor(parseFloat(amount) * Number(LAMPORTS_PER_SOL));
      const response = await fetch(`${BACKEND_URL}/api/deposit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ amount: lamports, depositor: walletAddress })
      });
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to generate deposit');
      }
      const { depositNote, onChainData }: DepositApiResponse = await response.json();
      console.log('[Deposit] Generated note:', depositNote.commitment.slice(0, 20) + '...');
      setStatus('Submitting to blockchain...');

      const [poolPda] = await getProgramDerivedAddress({
        programAddress: PRIVATE_TRANSFER_PROGRAM_ADDRESS,
        seeds: [getBytesEncoder().encode(SEEDS.POOL)]
      });
      const [poolVaultPda] = await getProgramDerivedAddress({
        programAddress: PRIVATE_TRANSFER_PROGRAM_ADDRESS,
        seeds: [getBytesEncoder().encode(SEEDS.VAULT), getAddressEncoder().encode(poolPda)]
      });
      const instructionData = getDepositInstructionDataEncoder().encode({
        commitment: new Uint8Array(onChainData.commitment),
        newRoot: new Uint8Array(onChainData.newRoot),
        amount: BigInt(onChainData.amount),
      });
      const instruction = {
        programAddress: PRIVATE_TRANSFER_PROGRAM_ADDRESS,
        accounts: [
          { address: poolPda, role: 1 },
          { address: poolVaultPda, role: 1 },
          { address: walletAddress, role: 3 },
          { address: SYSTEM_PROGRAM_ID, role: 0 },
        ],
        data: instructionData
      }
      setStatus('Please sign in your wallet...');

      try {
        const result = await sendTransaction({ instructions: [instruction] });
        if (result) {
          console.log('[Deposit] Success:', result);
          setStatus('');
          handleDepositComplete(depositNote);
        } else {
          throw new Error('Transaction failed');
        }
      } catch (txError) {
        throw txError;
      }
    } catch (error) {
      console.error('[Deposit] Error:', error);
      setStatus(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="card p-6">
      <div className="flex items-center gap-3 mb-6">
        <IconDeposit />
        <h2 className="text-white text-lg font-semibold">Deposit</h2>
      </div>
      {depositNote ? (
        <div className="space-y-4">
          <div className="p-4 deposit-note-box">
            <div className="flex items-center justify-between mb-3">
              <span className="text-[#14f195] text-sm font-medium">Deposit Note</span>
              <button onClick={handleCopy} className={`copy-btn ${copied ? 'copied' : ''}`}>
                {copied ? (
                  <>
                    <CheckIcon className="w-3.5 h-3.5" />
                    Copied
                  </>
                ) : (
                  <>
                    <CopyIcon className="w-3.5 h-3.5" />
                    Copy
                  </>
                )}
              </button>
            </div>
            <div className="bg-[#0a0b0d] rounded-lg p-3 font-mono text-xs text-[#8b8d94] break-all leading-relaxed max-h-32 overflow-y-auto">
              {JSON.stringify(depositNote, null, 0)}
            </div>
          </div>
          <div className="bg-[#14f195]/5 border border-[#14f195]/10 p-4 rounded-xl">
            <div className="flex items-start gap-3">
              <IconWarn />
              <div>
                <p className="text-[#14f195] text-sm font-medium">Save this note!</p>
                <p className="text-[#8b8d94] text-xs mt-1">You'll need it to withdraw. Copy and store it securely.</p>
              </div>
            </div>
          </div>

          <button onClick={handleClearNote} className="btn btn-outline w-full">
            Make Another Deposit
          </button>
        </div>
      ) : (
        <div className="space-y-5">
          <div>
            <label className="block text-sm font-medium mb-2 text-[#8b8d94]">Amount</label>
            <div className="relative">
              <input
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                min="0.001"
                step="0.01"
                placeholder="0.00"
                disabled={isProcessing}
              />
              <span className="absolute right-4 top-1/2 -translate-y-1/2 text-sm font-medium text-[#5c5e66]">SOL</span>
            </div>
            <p className="text-[#5c5e66] text-xs mt-2">Minimum: 0.001 SOL</p>
          </div>

          {status && (status.includes('Error') ? (
            <div className="p-3 rounded-lg text-sm status-error">{status}</div>
          ) : (
            <div className="p-3 rounded-lg text-sm status-info">{status}</div>
          ))}

          <button
            onClick={handleDeposit}
            disabled={!walletAddress || isProcessing || !amount}
            className="btn w-full btn-success"
          >
            {isProcessing ? (
              <span className="flex items-center gap-2">
                <AnimateSpin />
                Processing...
              </span>
            ) : (`Deposit ${amount || '0'} SOL`)}
          </button>
        </div>
      )}
    </div>
  )
}