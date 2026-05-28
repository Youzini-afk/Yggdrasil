import { allowedSurfaceCapabilityIdsForTest, summarizeConsoleDiagnostics } from "./project-frame";
import type { PackageRecord, SurfaceContributionRecord } from "@/protocol/client";

function assertEqual<T>(actual: T, expected: T) {
  if (actual !== expected) {
    throw new Error(`expected ${String(expected)}, got ${String(actual)}`);
  }
}

function assertDeepEqual(actual: unknown, expected: unknown) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

const contribution: SurfaceContributionRecord = {
  package_id: "pkg/surface",
  entry_kind: "subprocess",
  package_state: "ready",
  surface: {
    id: "pkg/surface/entry",
    version: "1",
    slot: "experience_entry",
    title: "Entry",
    capability_id: "pkg/surface/render",
    allowed_capability_ids: ["pkg/surface/exact_extra"],
    activation: {
      launch_capability_id: "pkg/surface/launch",
    },
    required_permissions: [],
    metadata: {
      requested_capabilities: ["attacker/metadata_grant"],
      nested: { capability_id: "attacker/nested_metadata_grant" },
    },
  },
};

assertDeepEqual([...allowedSurfaceCapabilityIdsForTest(contribution)].sort(), [
  "pkg/surface/exact_extra",
  "pkg/surface/launch",
  "pkg/surface/render",
]);

assertEqual(allowedSurfaceCapabilityIdsForTest(contribution).has("attacker/metadata_grant"), false);
assertEqual(allowedSurfaceCapabilityIdsForTest(contribution).has("attacker/nested_metadata_grant"), false);
assertEqual(allowedSurfaceCapabilityIdsForTest(contribution).has("pkg/surface/unrelated"), false);

assertDeepEqual([...allowedSurfaceCapabilityIdsForTest(null)], []);

const packages = [
  {
    id: "pkg/ready",
    version: "1",
    state: "ready",
    entry_kind: "subprocess",
    capability_count: 2,
    hook_count: 0,
  },
  {
    id: "pkg/degraded",
    version: "1",
    state: "degraded",
    entry_kind: "subprocess",
    capability_count: 1,
    hook_count: 1,
    last_failure: {
      package_id: "pkg/degraded",
      reason: "boom",
      failed_at: "now",
      stderr_tail_redacted: [],
      log_tail_redacted: [],
      stderr_truncated: false,
      redaction_state: "redacted",
      state: "degraded",
    },
  },
] satisfies PackageRecord[];

assertDeepEqual(summarizeConsoleDiagnostics({
  packages,
  events: [{
    id: "evt-1",
    session_id: "sess-1",
    sequence: 1,
    writer_package_id: "pkg/ready",
    kind: "kernel/v1/package.ready",
    payload: {},
    metadata: {},
    created_at: "now",
  }],
  updates: {
    results: [
      { package_id: "pkg/ready", status: "current", available: false },
      { package_id: "pkg/degraded", status: "repair_required", available: true },
    ],
  },
  errors: [],
  refreshedAt: "now",
}), {
  packageTotal: 2,
  packageHealthy: 1,
  packageProblem: 1,
  recentEvents: 1,
  updateAvailable: 1,
  updateChecked: true,
});

assertDeepEqual(summarizeConsoleDiagnostics(null), {
  packageTotal: 0,
  packageHealthy: 0,
  packageProblem: 0,
  recentEvents: 0,
  updateAvailable: 0,
  updateChecked: false,
});
