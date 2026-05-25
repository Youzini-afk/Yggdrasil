import { type ReactNode } from "react";
import { MotionConfig } from "motion/react";
import { IconContext } from "@phosphor-icons/react";
import { ThemeProvider } from "@/lib/theme";
import { LocaleProvider } from "@/lib/locale";
import { AuthProvider, useAuth } from "@/lib/auth-gate";
import { KernelProvider } from "@/lib/kernel-client";
import { ToastProvider } from "@/components/ui/toast";
import { TooltipProvider } from "@/components/ui/tooltip";
import { AuthGateScreen, AuthChecking } from "@/components/auth-gate";
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
      <LocaleProvider>
        <IconContext.Provider value={iconDefaults}>
          <AuthProvider>
            <AppInner>{children}</AppInner>
          </AuthProvider>
        </IconContext.Provider>
      </LocaleProvider>
    </ThemeProvider>
  );
}

function AppInner({ children }: { children?: ReactNode }) {
  const { status, token } = useAuth();

  if (status === "checking") {
    return <AuthChecking />;
  }

  const showGate = status === "required" || status === "invalid";
  if (showGate) {
    return <AuthGateScreen />;
  }

  return (
    <KernelProvider accessToken={token}>
      <MotionConfig reducedMotion="user">
        <TooltipProvider>
          <ToastProvider>{children ?? <Shell />}</ToastProvider>
        </TooltipProvider>
      </MotionConfig>
    </KernelProvider>
  );
}
