import { address, getAddressEncoder, getBytesEncoder, getProgramDerivedAddress } from "@solana/kit";
import { useSendTransaction, useWalletConnection } from "@solana/react-hooks";
import { useEffect, useState } from "react";
import { BACKEND_URL, SEEDS, SUNSPOT_VERIFIER_ID, SYSTEM_PROGRAM_ID } from "../constant";
import { getWithdrawInstructionDataEncoder, PRIVATE_TRANSFER_PROGRAM_ADDRESS } from "../generated/src/generated";
import { DepositNote, WithdrawApiResponse } from "../types";
import { formatSol, hexToBytes } from "../utils";
import { AnimateSpin, IconWithdraw } from "./Icon";

export default function Withdraw() {
  const { wallet } = useWalletConnection();
  const { send: sendTransaction, isSending } = useSendTransaction();

  const [depositNoteInput, setDepositNoteInput] = useState('');
  const [recipient, setRecipient] = useState('');
  const [status, setStatus] = useState('');
  const [loading, setLoading] = useState(false);
  const [parsedNote, setParsedNote] = useState<DepositNote | null>(null);

  const walletAddress = wallet?.account.address;

  // 每当列表中的变量发生变化时，执行方法
  useEffect(() => { walletAddress && !recipient && setRecipient(walletAddress) }, [walletAddress, recipient]);
  useEffect(() => {
    if (!depositNoteInput.trim()) {
      setParsedNote(null);
      return;
    }
    try {
      const parsed = JSON.parse(depositNoteInput);
      if (parsed.nullifier && parsed.secret && parsed.commitment && parsed.amount) {
        setParsedNote(parsed);
      } else {
        setParsedNote(null);
      }
    } catch (e) {
      setParsedNote(null);
    }
  }, [depositNoteInput]);

  const handleWithdraw = async () => {
    if (!walletAddress || !wallet || !parsedNote || !recipient) return;
    setLoading(true);
    setStatus('Generating ZK proof...');
    try {
      const response = await fetch(`${BACKEND_URL}/api/withdraw`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ depositNote: parsedNote, recipient, payer: walletAddress })
      });
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to generate proof');
      }
      const { withdrawalProof }: WithdrawApiResponse = await response.json();
      console.log('[Withdraw] Proof generated:', withdrawalProof.proof.length, 'bytes');
      setStatus('Submitting to blockchain...');

      const [poolPda] = await getProgramDerivedAddress({
        programAddress: PRIVATE_TRANSFER_PROGRAM_ADDRESS,
        seeds: [getBytesEncoder().encode(SEEDS.POOL)]
      });
      const [poolVaultPda] = await getProgramDerivedAddress({
        programAddress: PRIVATE_TRANSFER_PROGRAM_ADDRESS,
        seeds: [getBytesEncoder().encode(SEEDS.VAULT), getAddressEncoder().encode(poolPda)]
      });
      const [nullifierSetPda] = await getProgramDerivedAddress({
        programAddress: PRIVATE_TRANSFER_PROGRAM_ADDRESS,
        seeds: [getBytesEncoder().encode(SEEDS.NULLIFIERS), getAddressEncoder().encode(poolPda)],
      });
      const instructionData = getWithdrawInstructionDataEncoder().encode({
        proof: new Uint8Array(withdrawalProof.proof),
        nullifierHash: hexToBytes(withdrawalProof.nullifierHash),
        root: hexToBytes(withdrawalProof.merkleRoot),
        recipient: address(withdrawalProof.recipient),
        amount: BigInt(withdrawalProof.amount)
      });
      const instruction = {
        programAddress: PRIVATE_TRANSFER_PROGRAM_ADDRESS,
        accounts: [
          { address: poolPda, role: 1 },
          { address: nullifierSetPda, role: 1 },
          { address: poolVaultPda, role: 1 },
          { address: address(withdrawalProof.recipient), role: 1 },
          { address: SUNSPOT_VERIFIER_ID, role: 0 },
          { address: SYSTEM_PROGRAM_ID, role: 0 },
        ],
        data: instructionData
      }
      setStatus('Please sign in your wallet...');

      try {
        const result = await sendTransaction({ instructions: [instruction] });
        if (result) {
          console.log('[Withdraw] Success:', result);
          setStatus(`Success! ${formatSol(BigInt(withdrawalProof.amount))} SOL sent to ${recipient.slice(0, 8)}...`);
          setDepositNoteInput('');
          setParsedNote(null);
        } else {
          throw new Error('Transaction failed')
        }
      } catch (txError) {
        throw txError;
      }
    } catch (error) {
      console.error('[Withdraw] Error:', error);
      setStatus(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  }

  const depositAmountSol = parsedNote ? formatSol(BigInt(parsedNote.amount)) : '0';
  const isProcessing = loading || isSending;

  return (
    <div className="card p-6">
      <div className="flex items-center gap-3 mb-6">
        <IconWithdraw />
        <h2 className="text-lg font-semibold text-white">Withdraw</h2>
      </div>
      <div className="space-y-5">
        <div>
          <label className="text-[#8b8d94] font-medium text-sm block mb-2">Deposit Note</label>
          <textarea
            value={depositNoteInput}
            onChange={(e) => setDepositNoteInput(e.target.value)}
            placeholder="Paste your deposit note here..."
            rows={3}
            disabled={isProcessing}
            className="bg-[#0a0b0d] border border-[#232529] w-full rounded-lg p-3 font-mono text-xs text-[#f4f4f5] placeholder-[#5c5e66] resize-none focus:outline-none focus:border-[#9945ff] focus:ring-1 focus:ring-[#9945FF]/20 transition-all disabled:opacity-60"
          />
          {depositNoteInput && !parsedNote && (
            <p className="text-xs text-red-400 mt-2">Invalid deposit note format</p>
          )}
          {parsedNote && (
            <p>Valid note: {depositAmountSol} SOL</p>
          )}
        </div>
        <div>
          <label className="block text-sm font-medium text-[#8b8d94] mb-2">Recipient Address</label>
          <input
            type="text"
            value={recipient}
            onChange={(e) => setRecipient(e.target.value)}
            placeholder="SolanaAddress"
            className="font-mono text-sm"
            disabled={isProcessing}
          />
        </div>

        {status && (
          <div className={`p-3 rounded-lg text-sm ${status.includes('Error') ? 'status-error' : status.includes('Success') ? 'status-success' : 'status-info'
            }`}>
            {status}
          </div>
        )}

        <button
          onClick={handleWithdraw}
          disabled={!walletAddress || !parsedNote || !recipient || isProcessing}
          className="btn btn-primary w-full"
        >
          {isProcessing ? (
            <span className="flex items-center gap-2">
              <AnimateSpin />
              {loading ? 'Generating proof...' : 'Processing...'}
            </span>
          ) : parsedNote ? (`Withdraw ${depositAmountSol} SOL`) : ('Withdraw')}
        </button>
      </div>
    </div>
  )
}