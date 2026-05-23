# Yggdrasil TypeScript subprocess SDK

Thin helper for JSON-RPC-over-stdio capability packages. It wraps the public subprocess protocol: handshake, `capability.invoke`, and reverse public `kernel.v1.*` calls such as `kernel.v1.outbound.execute` / `kernel.v1.outbound.stream`.

```ts
import { serveSubprocessPackage } from "./index";

serveSubprocessPackage({
  onInvoke: ({ input }) => input ?? null,
});
```

The SDK does not expose kernel internals and should remain usable by official and third-party packages equally. Reverse calls are dispatched by the host with the caller principal locked to this subprocess package.
