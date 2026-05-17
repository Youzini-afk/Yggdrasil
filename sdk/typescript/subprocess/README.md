# Yggdrasil TypeScript subprocess SDK

Thin helper for JSON-RPC-over-stdio capability packages. It wraps only the public subprocess protocol: handshake and `capability.invoke`.

```ts
import { serveSubprocessPackage } from "./index";

serveSubprocessPackage({
  onInvoke: ({ input }) => input ?? null,
});
```

The SDK does not expose kernel internals and should remain usable by official and third-party packages equally.
