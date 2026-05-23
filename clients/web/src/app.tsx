import { type ReactNode } from "react";
import { MotionConfig } from "motion/react";
import { IconContext } from "@phosphor-icons/react";
import { ThemeProvider } from "@/lib/theme";
import { KernelProvider } from "@/lib/kernel-client";
import { ToastProvider } from "@/components/ui/toast";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Shell } from "@/components/layout/shell";

const iconDefaults = {
  color: "currentColor",
  size: 18,
  weight: "regular" as const,
  mirrored: false,
};

/**
 * `reducedMotion="user"` makes every Motion animation in the tree honor the
 * `prefers-reduced-motion` media query. Combined with the global CSS rule
 * in `styles/app.css`, motion is now consistently suppressed for users who
 * request it — no per-component `useReducedMotion` plumbing needed.
 */
export function App({ children }: { children?: ReactNode }) {
  return (
    <ThemeProvider>
      <IconContext.Provider value={iconDefaults}>
        <KernelProvider>
          <MotionConfig reducedMotion="user">
            <TooltipProvider>
              <ToastProvider>{children ?? <Shell />}</ToastProvider>
            </TooltipProvider>
          </MotionConfig>
        </KernelProvider>
      </IconContext.Provider>
    </ThemeProvider>
  );
}
