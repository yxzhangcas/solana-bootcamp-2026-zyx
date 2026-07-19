import { LAMPORTS_PER_SOL } from "../constant";

fetch(`http://localhost:4001/api/airdrop`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ address: 'GUaFpyxWonsCk94i4JHBBncaiTG4sVDFHdvRHiTdrFie', amount: Number(LAMPORTS_PER_SOL) })
}).then(console.log).catch(console.error);