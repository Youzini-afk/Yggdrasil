import { strict as assert } from "node:assert";
import { failureDetailFromPackage } from "./failure-diagnostics";
import type { PackageRecord, SubprocessLogLine } from "@/protocol/client";

const base: PackageRecord = {
  id: "pkg/demo",
  version: "1.0.0",
  state: "failed",
  entry_kind: "subprocess",
  capability_count: 0,
  hook_count: 0,
};

const rawLogs: SubprocessLogLine[] = [
  { package_id: "pkg/demo", stream: "stderr", line: "RAW_SECRET=sk-live" },
];

const withoutRedacted = failureDetailFromPackage("Demo", base, rawLogs);
assert.deepEqual(withoutRedacted.log, []);
assert.equal(withoutRedacted.logRedacted, false);

const withUnsafeFailure = failureDetailFromPackage(
  "Demo",
  {
    ...base,
    last_failure: {
      package_id: "pkg/demo",
      reason: "failed",
      failed_at: new Date().toISOString(),
      stderr_tail_redacted: [],
      log_tail_redacted: [],
      stderr_truncated: false,
      redaction_state: "not_captured",
      state: "failed",
    },
  },
  rawLogs,
);
assert.deepEqual(withUnsafeFailure.log, []);

const withRedacted = failureDetailFromPackage(
  "Demo",
  {
    ...base,
    last_failure: {
      package_id: "pkg/demo",
      reason: "failed",
      failed_at: new Date().toISOString(),
      stderr_tail_redacted: ["[REDACTED]"],
      log_tail_redacted: [],
      stderr_truncated: false,
      redaction_state: "redacted",
      state: "failed",
    },
  },
  rawLogs,
);
assert.deepEqual(withRedacted.log, ["[REDACTED]"]);
assert.equal(withRedacted.logRedacted, true);
