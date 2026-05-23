import { Bell, GearSix, Moon, Sun } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { useTheme } from "@/lib/theme";
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
  const { theme, toggle } = useTheme();
  const [, navigate] = useRoute();

  return (
    <header
      className={cn(
        "sticky top-0 z-30 flex h-[60px] items-center justify-between border-b border-whisper-border bg-warm-bone/85 px-4 backdrop-blur-[20px] sm:px-6 lg:px-8",
      )}
    >
      <nav className="flex min-w-0 items-center gap-3 text-[14px]">
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

      <div className="flex items-center gap-1">
        <Tooltip label="Notifications">
          <Button tone="icon" size="icon" aria-label="Notifications">
            <Bell size={18} />
            <span className="absolute right-2 top-2 size-1.5 rounded-full bg-aged-brass" aria-hidden />
          </Button>
        </Tooltip>
        <Tooltip label={theme === "dark" ? "Light mode" : "Dark mode"}>
          <Button tone="icon" size="icon" onClick={toggle} aria-label="Toggle theme">
            {theme === "dark" ? <Sun size={18} /> : <Moon size={18} />}
          </Button>
        </Tooltip>
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
    </header>
  );
}
