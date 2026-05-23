import { type ReactNode } from "react";
import { cn } from "@/lib/cn";

export function Eyebrow({ children, className }: { children: ReactNode; className?: string }) {
  return <p className={cn("eyebrow", className)}>{children}</p>;
}

export function EyebrowSm({ children, className }: { children: ReactNode; className?: string }) {
  return <p className={cn("eyebrow-sm", className)}>{children}</p>;
}

export function PageTitle({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <h1
      className={cn(
        "font-display font-bold leading-[1.1] tracking-[-0.02em] text-charcoal-ink",
        "text-[clamp(1.875rem,3vw,2.5rem)]",
        className,
      )}
    >
      {children}
    </h1>
  );
}

export function HeroTitle({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <h1
      className={cn(
        "font-display font-bold leading-[1.05] tracking-[-0.025em] text-charcoal-ink",
        "text-[clamp(2.5rem,5vw,4rem)]",
        className,
      )}
    >
      {children}
    </h1>
  );
}

export function CardTitle({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <h3
      className={cn(
        "font-display text-[18px] font-bold leading-tight tracking-[-0.015em] text-charcoal-ink",
        className,
      )}
    >
      {children}
    </h3>
  );
}

export function Mono({ children, className }: { children: ReactNode; className?: string }) {
  return <span className={cn("font-mono text-[12px] text-charcoal-ink", className)}>{children}</span>;
}
