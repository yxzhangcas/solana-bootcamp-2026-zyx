export interface DepositNote {
  nullifier: string;
  secret: string;
  amount: string;
  commitment: string;
  nullifierHash: string;
  merkleRoot: string;
  leafIndex: number;
  timestamp: number;
}

export interface OnChainData {
  commitment: number[];
  newRoot: number[];
  amount: string;
}

export interface WithdrawalProof {
  proof: number[];
  publicWitness: number[];
  nullifierHash: string;
  merkleRoot: string;
  recipient: string;
  amount: string;
}

export interface DepositApiResponse {
  depositNote: DepositNote;
  onChainData: OnChainData;
}

export interface WithdrawApiResponse {
  withdrawalProof: WithdrawalProof;
}