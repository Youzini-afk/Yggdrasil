import { Bell, GearSix, Moon, SignOut, Sun } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { useTheme } from "@/lib/theme";
import { useAuth } from "@/lib/auth-gate";
import { useRoute, type Route } from "@/lib/router";
import { cn } from "@/lib/cn";

const breadcrumbForRoute = (route: Route): string => {
  switch (route.kind) {
    case "home":
      return "Home";
    case "settings":
      return "Settings";
    case "project":
      return `Projects / ${route.projectId}`;
  }
};

export function PlatformTopbar({ route }: { route: Route }) {
  const { theme, preference, setPreference } = useTheme();
  const { token, logout } = useAuth();
  const [, navigate] = useRoute();

  // Cycle: system → light → dark → system. Preserves the user's choice to
  // follow the OS instead of forcing it off after the first toggle.
  const cycleTheme = () => {
    const next =
      preference === "system" ? "light" : preference === "light" ? "dark" : "system";
    setPreference(next);
  };

  return (
    <header
      className={cn(
        "sticky top-0 z-30 flex h-[60px] items-center justify-between border-b border-whisper-border bg-warm-bone/85 px-3 backdrop-blur-[20px] sm:px-6 lg:px-8",
      )}
    >
      <nav className="flex min-w-0 items-center gap-2 text-[14px] sm:gap-3">
        <button
          type="button"
          onClick={() => navigate({ kind: "home" })}
          className="font-display text-[18px] font-bold leading-none tracking-[-0.015em] text-charcoal-ink hover:text-aged-brass-deep"
        >
          Yggdrasil
        </button>
        <span className="hidden text-muted-tone sm:inline" aria-hidden>
          /
        </span>
        <span className="hidden truncate text-[13px] text-steel-secondary sm:inline">
          {breadcrumbForRoute(route)}
        </span>
      </nav>

      <div className="flex shrink-0 items-center gap-0.5 sm:gap-1">
        <Tooltip label="Notifications">
          <Button tone="icon" size="icon" aria-label="Notifications" className="relative">
            <Bell size={18} />
          </Button>
        </Tooltip>
        <Tooltip
          label={
            preference === "system"
              ? `System (${theme === "dark" ? "Dark" : "Light"})`
              : preference === "light"
                ? "Light mode"
                : "Dark mode"
          }
        >
          <Button
            tone="icon"
            size="icon"
            onClick={cycleTheme}
            aria-label={`Theme preference: ${preference}`}
            aria-pressed={preference === "dark"}
          >
            {/* Cycle indicator: filled icon for explicit choice, outline for system. */}
            {preference === "system" ? (
              theme === "dark" ? <Moon size={18} /> : <Sun size={18} />
            ) : preference === "dark" ? (
              <Moon size={18} weight="fill" />
            ) : (
              <Sun size={18} weight="fill" />
            )}
          </Button>
        </Tooltip>
        <div className="hidden sm:inline">
          <Tooltip label="Settings">
            <Button
              tone="icon"
              size="icon"
              aria-label="Settings"
              onClick={() => navigate({ kind: "settings", tab: "api-connections" })}
            >
              <GearSix size={18} />
            </Button>
          </Tooltip>
        </div>
        {token ? (
          <Tooltip label="Log out">
            <Button
              tone="icon"
              size="icon"
              aria-label="Log out"
              onClick={logout}
              className="text-deep-rust hover:bg-deep-rust-surface"
            >
              <SignOut size={18} />
            </Button>
          </Tooltip>
        ) : null}
      </div>
    </header>
  );
}
