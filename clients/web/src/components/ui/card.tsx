import { forwardRef } from "react";
import { cn } from "@/lib/cn";

export const Card = forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(
        "rounded-[20px] border border-whisper-border bg-pure-surface shadow-card",
        className,
      )}
      {...props}
    />
  ),
);
Card.displayName = "Card";

export const CardSection = forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { divided?: boolean }
>(({ className, divided, ...props }, ref) => (
  <div
    ref={ref}
    className={cn(
      "px-6 py-5",
      divided && "border-t border-whisper-border first:border-t-0",
      className,
    )}
    {...props}
  />
));
CardSection.displayName = "CardSection";

export const CardRow = forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { divided?: boolean }
>(({ className, divided = true, ...props }, ref) => (
  <div
    ref={ref}
    className={cn(
      "flex items-center gap-3 py-3",
      divided && "border-t border-whisper-border first:border-t-0",
      className,
    )}
    {...props}
  />
));
CardRow.displayName = "CardRow";
