export function Footer() {
  return (
    <footer className="border-t border-border-low mt-16">
      <div className="mx-auto max-w-5xl px-4 py-6">
        <div className="flex flex-wrap items-center justify-between gap-4 text-xs text-muted">
          <div className="flex items-center gap-2">
            <span className="rounded bg-yellow-100 px-2 py-0.5 text-yellow-800 font-medium">
              Devnet
            </span>
            <span>Built with Anchor + @solana/kit</span>
          </div>
          <div className="flex gap-4">
            <a
              href="https://www.anchor-lang.com/docs"
              target="_blank"
              rel="noreferrer"
              className="hover:text-foreground transition"
            >
              Anchor Docs
            </a>
            <a
              href="https://solana.com/docs"
              target="_blank"
              rel="noreferrer"
              className="hover:text-foreground transition"
            >
              Solana Docs
            </a>
            <a
              href="https://faucet.solana.com/"
              target="_blank"
              rel="noreferrer"
              className="hover:text-foreground transition"
            >
              Faucet
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
}
