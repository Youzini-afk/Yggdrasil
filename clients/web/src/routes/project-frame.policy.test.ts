import { allowedSurfaceCapabilityIdsForTest } from "./project-frame";
import type { SurfaceContributionRecord } from "@/protocol/client";

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
