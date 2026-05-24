import { strict as assert } from "node:assert";
import {
  callSurfaceBridgeForTest,
  canSubscribeSurfaceStreamForTest,
  createSurfaceBridgeState,
  SurfaceBridgeError,
  type SurfaceHostBridge,
} from "./surface-host";

async function rejectsWithCode(promise: Promise<unknown>, code: string) {
  await assert.rejects(promise, (err) => err instanceof SurfaceBridgeError && err.code === code);
}

const calls: Array<{ method: string; params: unknown }> = [];
const bridge: SurfaceHostBridge = {
  currentSessionId: "session-current",
  allowedCapabilityIds: ["pkg/cap"],
  callRpc: async (method, params) => {
    calls.push({ method, params });
    if (method === "kernel.v1.capability.stream") {
      return { invocation: { invocation_id: "inv-1", stream_id: "stream-1" } };
    }
    return { ok: true };
  },
};

await rejectsWithCode(
  callSurfaceBridgeForTest(bridge, { id: "1", method: "kernel.v1.install.execute", params: {} }),
  "rpc_denied",
);

await callSurfaceBridgeForTest(bridge, {
  id: "2",
  method: "kernel.v1.capability.invoke",
  params: { capability_id: "pkg/cap", session_id: "attacker-session", input: { hello: "world" } },
});
assert.equal(calls.at(-1)?.method, "kernel.v1.capability.invoke");
assert.equal((calls.at(-1)?.params as { session_id?: string }).session_id, "session-current");

await rejectsWithCode(
  callSurfaceBridgeForTest(bridge, {
    id: "2b",
    method: "kernel.v1.capability.invoke",
    params: { capability_id: "pkg/unrelated", input: {} },
  }),
  "capability_denied",
);

const state = createSurfaceBridgeState();
await callSurfaceBridgeForTest(
  bridge,
  { id: "3", method: "kernel.v1.capability.stream", params: { capability_id: "pkg/cap", session_id: "other" } },
  state,
);
assert.deepEqual(canSubscribeSurfaceStreamForTest("sub-1", "stream-unknown", [], ["stream-1"]), {
  ok: false,
  code: "not_owned",
});
assert.deepEqual(canSubscribeSurfaceStreamForTest("sub-1", "stream-1", ["sub-1"], ["stream-1"]), {
  ok: false,
  code: "duplicate_subscription",
});

await rejectsWithCode(
  callSurfaceBridgeForTest(
    bridge,
    { id: "4", method: "kernel.v1.capability.cancel", params: { stream_id: "not-owned" } },
    state,
  ),
  "not_owned",
);

await callSurfaceBridgeForTest(
  bridge,
  { id: "5", method: "kernel.v1.capability.cancel", params: { stream_id: "stream-1" } },
  state,
);
assert.equal((calls.at(-1)?.params as { session_id?: string }).session_id, "session-current");
