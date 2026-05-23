import { forwardRef } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/cn";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 font-medium select-none transition-[transform,background,border-color,color] outline-none focus-visible:ring-2 focus-visible:ring-aged-brass focus-visible:ring-offset-2 focus-visible:ring-offset-warm-bone disabled:pointer-events-none disabled:opacity-50 active:translate-y-px",
  {
    variants: {
      tone: {
        primary:
          "bg-aged-brass text-white hover:bg-aged-brass-deep",
        secondary:
          "bg-transparent text-charcoal-ink border border-whisper-border hover:bg-whisper-border-strong/30",
        tertiary:
          "bg-transparent text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass",
        destructive:
          "bg-transparent text-deep-rust border border-deep-rust hover:bg-deep-rust-surface",
        icon:
          "bg-transparent text-charcoal-ink hover:bg-whisper-border-strong/40 active:translate-y-0",
      },
      size: {
        sm: "h-8 px-3 text-[12px] rounded-[8px]",
        md: "h-10 px-4 text-[13px] rounded-[10px]",
        lg: "h-11 px-5 text-[14px] rounded-[10px]",
        icon: "size-9 rounded-[10px]",
        "icon-sm": "size-7 rounded-[8px]",
      },
    },
    defaultVariants: {
      tone: "secondary",
      size: "md",
    },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, tone, size, ...props }, ref) => (
    <button ref={ref} className={cn(buttonVariants({ tone, size }), className)} {...props} />
  ),
);
Button.displayName = "Button";

export { buttonVariants };
