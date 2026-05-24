import { strict as assert } from "node:assert";
import { allowedSurfaceCapabilityIdsForTest } from "./project-frame";
import type { SurfaceContributionRecord } from "@/protocol/client";

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

assert.deepEqual([...allowedSurfaceCapabilityIdsForTest(contribution)].sort(), [
  "pkg/surface/launch",
  "pkg/surface/render",
]);

assert.deepEqual([...allowedSurfaceCapabilityIdsForTest(null)], []);
