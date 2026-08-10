const HOW_IT_WORKS_STEPS = [
  {
    title: "Create",
    description:
      "Anyone can create a market with a yes/no question and deadline.",
  },
  { title: "Bet", description: "Stake SOL on YES or NO before the deadline." },
  {
    title: "Resolve",
    description: "After the deadline, the creator declares the outcome.",
  },
  {
    title: "Claim",
    description: "Winners split the losing pool proportionally.",
  },
];

export function WorkDescription() {
  return (
    <details className="mt-12 rounded-lg border border-border-low">
      <summary className="cursor-pointer px-4 py-3 text-sm font-medium hover:bg-cream/30">
        How it works
      </summary>
      <div className="border-t border-border-low px-4 py-4 text-sm text-muted space-y-3">
        {HOW_IT_WORKS_STEPS.map((step, index) => (
          <div key={step.title} className="flex gap-3">
            <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-cream text-xs font-medium">
              {index + 1}
            </span>
            <p>
              <strong className="text-foreground">{step.title}</strong> -{" "}
              {step.description}
            </p>
          </div>
        ))}
      </div>
    </details>
  );
}
