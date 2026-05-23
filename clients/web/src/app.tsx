import { type ReactNode } from "react";
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

export function App({ children }: { children?: ReactNode }) {
  return (
    <ThemeProvider>
      <IconContext.Provider value={iconDefaults}>
        <KernelProvider>
          <TooltipProvider>
            <ToastProvider>{children ?? <Shell />}</ToastProvider>
          </TooltipProvider>
        </KernelProvider>
      </IconContext.Provider>
    </ThemeProvider>
  );
}
