# @yggdrasil/kernel-sdk

Generated TypeScript bindings for the Yggdrasil kernel contract.

```ts
import { attach, fromHttpRpc } from "@yggdrasil/kernel-sdk";

const client = attach(fromHttpRpc("http://localhost:8080/rpc"));
const info = await client.hostInfo({});
```

The generated types come from `docs/spec/v1/schemas/`. Regenerate with:

```bash
bash scripts/regen-sdks.sh
```

This package can be consumed from npm or via a workspace path reference:

```json
{ "dependencies": { "@yggdrasil/kernel-sdk": "file:../yggdrasil/sdk/typescript/kernel-sdk" } }
```
