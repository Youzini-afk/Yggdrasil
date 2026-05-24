import { useCallback, useEffect } from "react";
import type { FailureDetail } from "@/components/install/failure-modal";
import type { ToastInput } from "@/components/ui/toast";
import type { YggProtocolClient, PackageRecord, ProjectRecord } from "@/protocol/client";
import {
  failureDetailFromPackage,
  noFailureDiagnostic,
  resolvePackageStatus,
} from "./failure-diagnostics";

interface UseProjectActionsArgs {
  client: YggProtocolClient;
  onLaunch: (projectId: string) => void;
  pushToast: (toast: ToastInput) => string;
  refreshProjects: () => void;
  setShowInstall: (show: boolean) => void;
  setFailureProjectId: (projectId: string | null) => void;
  setFailureDetail: (detail: FailureDetail | undefined) => void;
}

export function useProjectActions({
  client,
  onLaunch,
  pushToast,
  refreshProjects,
  setShowInstall,
  setFailureProjectId,
  setFailureDetail,
}: UseProjectActionsArgs) {
  // Cmd/Ctrl + N opens the install modal.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "n") {
        event.preventDefault();
        setShowInstall(true);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [setShowInstall]);

  const onStop = useCallback(
    async (projectId: string, title: string) => {
      try {
        await client.stopProject(projectId);
        pushToast({ variant: "success", title: `Stopped ${title}` });
        refreshProjects();
      } catch (err) {
        pushToast({
          variant: "error",
          title: "Stop failed",
          body: "The project could not be stopped. Check the local host and try again.",
        });
      }
    },
    [client, pushToast, refreshProjects],
  );

  const onUninstall = useCallback(
    (title: string) => {
      pushToast({
        variant: "info",
        title: `Uninstall ${title}`,
        body: `Confirm in CLI: yg uninstall ${title}`,
      });
    },
    [pushToast],
  );

  const onInstallClick = useCallback(() => setShowInstall(true), [setShowInstall]);

  const onShowFailure = useCallback(
    async (project: ProjectRecord) => {
      setFailureProjectId(project.id);
      setFailureDetail({
        projectName: project.title,
        title: "Loading diagnostics…",
        summary: "Reading bounded package failure details from the kernel.",
      });
      try {
        const descriptor = await client.getProject(project.id);
        const packageIds = descriptor.packages ?? [];
        const knownPackages = await client.packages().catch<PackageRecord[]>(() => []);
        const packageLookup = new Map(knownPackages.map((record) => [record.id, record]));
        if (packageIds.length === 0) {
          setFailureDetail(noFailureDiagnostic(project.title, "Project descriptor does not list packages."));
          return;
        }
        const records = (
          await Promise.all(packageIds.map((packageId) => resolvePackageStatus(client, packageId, packageLookup)))
        ).filter((record): record is PackageRecord => Boolean(record));
        const failed = records.find((record) => record.last_failure) ?? records.find((record) => record.state === "degraded") ?? records[0];
        if (!failed) {
          setFailureDetail(noFailureDiagnostic(project.title, "No associated package status was available."));
          return;
        }
        setFailureDetail(failureDetailFromPackage(project.title, failed, []));
      } catch (err) {
        setFailureDetail(noFailureDiagnostic(project.title, "Diagnostics are unavailable. Try again from the local UI."));
      }
    },
    [client, setFailureDetail, setFailureProjectId],
  );

  const onCardLaunch = useCallback(
    (project: ProjectRecord) => {
      if (project.state === "failed") {
        void onShowFailure(project);
        return;
      }
      onLaunch(project.id);
    },
    [onLaunch, onShowFailure],
  );

  return { onStop, onUninstall, onInstallClick, onShowFailure, onCardLaunch };
}
