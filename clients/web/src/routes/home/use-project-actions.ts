import { useCallback, useEffect } from "react";
import type { FailureDetail } from "@/components/install/failure-modal";
import type { ToastInput } from "@/components/ui/toast";
import type { YggProtocolClient, PackageRecord, ProjectRecord } from "@/protocol/client";
import {
  failureDetailFromPackage,
  noFailureDiagnostic,
  resolvePackageStatus,
} from "./failure-diagnostics";
import { recordOpen } from "./use-recently-opened";

interface UseProjectActionsArgs {
  client: YggProtocolClient;
  onLaunch: (projectId: string) => void;
  pushToast: (toast: ToastInput) => string;
  refreshProjects: () => void;
  setShowInstall: (show: boolean) => void;
  setFailureProjectId: (projectId: string | null) => void;
  setFailureDetail: (detail: FailureDetail | undefined) => void;
  labels: {
    stoppedToast: (title: string) => string;
    stopFailedTitle: string;
    stopFailedBody: string;
    uninstallTitle: (title: string) => string;
    uninstallBody: (title: string) => string;
    loadingDiagnostics: string;
    loadingDiagnosticsSummary: string;
    descriptorNoPackages: string;
    noPackageStatus: string;
    diagnosticsUnavailable: string;
  };
}

export function useProjectActions({
  client,
  onLaunch,
  pushToast,
  refreshProjects,
  setShowInstall,
  setFailureProjectId,
  setFailureDetail,
  labels,
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
        pushToast({ variant: "success", title: labels.stoppedToast(title) });
        refreshProjects();
      } catch (err) {
        pushToast({
          variant: "error",
          title: labels.stopFailedTitle,
          body: labels.stopFailedBody,
        });
      }
    },
    [client, labels, pushToast, refreshProjects],
  );

  const onUninstall = useCallback(
    (title: string) => {
      pushToast({
        variant: "info",
        title: labels.uninstallTitle(title),
        body: labels.uninstallBody(title),
      });
    },
    [labels, pushToast],
  );

  const onInstallClick = useCallback(() => setShowInstall(true), [setShowInstall]);

  const onShowFailure = useCallback(
    async (project: ProjectRecord) => {
      setFailureProjectId(project.id);
      setFailureDetail({
        projectName: project.title,
        title: labels.loadingDiagnostics,
        summary: labels.loadingDiagnosticsSummary,
      });
      try {
        const descriptor = await client.getProject(project.id);
        const packageIds = descriptor.packages ?? [];
        const knownPackages = await client.packages().catch<PackageRecord[]>(() => []);
        const packageLookup = new Map(knownPackages.map((record) => [record.id, record]));
        if (packageIds.length === 0) {
          setFailureDetail(noFailureDiagnostic(project.title, labels.descriptorNoPackages));
          return;
        }
        const records = (
          await Promise.all(packageIds.map((packageId) => resolvePackageStatus(client, packageId, packageLookup)))
        ).filter((record): record is PackageRecord => Boolean(record));
        const failed = records.find((record) => record.last_failure) ?? records.find((record) => record.state === "degraded") ?? records[0];
        if (!failed) {
          setFailureDetail(noFailureDiagnostic(project.title, labels.noPackageStatus));
          return;
        }
        setFailureDetail(failureDetailFromPackage(project.title, failed, []));
      } catch (err) {
        setFailureDetail(noFailureDiagnostic(project.title, labels.diagnosticsUnavailable));
      }
    },
    [client, labels, setFailureDetail, setFailureProjectId],
  );

  const onCardLaunch = useCallback(
    (project: ProjectRecord) => {
      recordOpen(project.id);
      if (project.state === "failed") {
        void onShowFailure(project);
        return;
      }
      onLaunch(project.id);
    },
    [onLaunch, onShowFailure, recordOpen],
  );

  return { onStop, onUninstall, onInstallClick, onShowFailure, onCardLaunch };
}
