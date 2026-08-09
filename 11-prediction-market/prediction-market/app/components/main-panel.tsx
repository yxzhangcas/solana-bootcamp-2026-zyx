"use client";

export function MainPanel() {
  return (
    <div className="flex flex-col gap-6 md:flex-row md:items-center md:justify-between">
      <div>
        <h1 className="font-black tracking-tight text-foreground">
          <span className="block text-6xl md:text-7xl">Anchor</span>
          <span className="block text-7xl md:text-8xl">Vault</span>
        </h1>
      </div>

      <div className="flex max-w-2xl flex-col gap-3">
        <p className="text-base leading-relaxed text-foreground/50">
          This program creates a personal vault for each user using a Program
          Derived Address (PDA). Connect your wallet, deposit SOL into your
          vault, and withdraw it anytime. Only you can access your funds.
        </p>
        <p className="text-sm leading-relaxed text-foreground/40">
          The vault is an{" "}
          <a
            href="https://www.anchor-lang.com/docs/introduction"
            target="_blank"
            rel="noopener noreferrer"
            className="underline underline-offset-2"
          >
            Anchor
          </a>{" "}
          program you can deploy to localnet or devnet and modify yourself.
          Check the README for setup instructions.
        </p>
        <div className="flex flex-wrap gap-4">
          <a
            href="https://solana.com/docs"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-sm font-medium text-foreground/70 underline underline-offset-4 transition-colors hover:text-foreground"
          >
            Solana docs
            <span aria-hidden="true">&rarr;</span>
          </a>
          <a
            href="https://www.anchor-lang.com/docs/introduction"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-sm font-medium text-foreground/70 underline underline-offset-4 transition-colors hover:text-foreground"
          >
            Anchor docs
            <span aria-hidden="true">&rarr;</span>
          </a>
          <a
            href="https://faucet.solana.com/"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-sm font-medium text-foreground/70 underline underline-offset-4 transition-colors hover:text-foreground"
          >
            Faucet
            <span aria-hidden="true">&rarr;</span>
          </a>
        </div>
      </div>
    </div>
  );
}
