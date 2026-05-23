import { useRoute, type Route } from "@/lib/router";
import { PlatformTopbar } from "@/components/layout/platform-topbar";
import { HomePage } from "@/routes/home";
import { SettingsPage } from "@/routes/settings";
import { ProjectFrame } from "@/routes/project-frame";

export function Shell() {
  const [route] = useRoute();

  return (
    <div className="flex min-h-[100dvh] flex-col bg-warm-bone text-charcoal-ink">
      <PlatformTopbar route={route} />
      <main className="flex flex-1 flex-col">{renderRoute(route)}</main>
    </div>
  );
}

function renderRoute(route: Route) {
  switch (route.kind) {
    case "home":
      return <HomePage />;
    case "settings":
      return <SettingsPage tab={route.tab} />;
    case "project":
      return <ProjectFrame projectId={route.projectId} />;
  }
}
