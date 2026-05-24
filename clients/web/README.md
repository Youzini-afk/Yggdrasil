# Yggdrasil platform shell (`clients/web`)

The Yggdrasil platform's user-facing chrome — Home, Settings, Install flow,
project frame, and toast/notification system. Built as a React 19 single-page
app with Tailwind v4. Styles, layout, and behavior follow the Editorial
Workshop design system in [`../../docs/design/PLATFORM_UI_DESIGN.md`](../../docs/design/PLATFORM_UI_DESIGN.md).

This client is the platform shell only. Project surfaces (YdlTavern, custom
projects, etc.) mount inside the project frame as iframes through `SurfaceHost`
and own their own visual identity; the shell does not impose a style on them.

---

## Quick start

```bash
# Run the host first (separate terminal)
cargo run -p ygg-cli -- host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml

# Run the web shell
npm install --prefix clients/web
npm run dev   --prefix clients/web   # 127.0.0.1:1420
npm run check --prefix clients/web   # tsc only
npm run build --prefix clients/web   # production bundle to dist/
```

For desktop builds wrap `dist/` with [`../desktop`](../desktop) (Tauri 2.x).

---

## Stack

- **React 19** with `createRoot` + StrictMode.
- **Tailwind v4** as the styling system. All design tokens defined in
  `src/styles/app.css` via the `@theme` directive — there is no `tailwind.config.js`.
  Custom `dark:` variant binds to `data-theme="dark"` on `<html>`.
- **Vite 6** for bundling, dev server, and the Surface bundle dev middleware.
- **Motion (formerly Framer Motion) v12** for entrance/exit transitions on
  modals, toasts, and timeline rows. Honors `prefers-reduced-motion`.
- **Radix UI** for `Dialog`, `DropdownMenu`, and `Tooltip` primitives — keeps
  focus-trap, keyboard handling, and ARIA correct without recreating those
  patterns.
- **Phosphor Icons** (regular weight, 16/18px). Re-exported from
  `src/components/icons.tsx` so component code uses semantic names.
- **CVA + clsx + tailwind-merge** for variant-driven components (Button,
  StatusPill).
- **Variable fonts**: `@fontsource-variable/{bricolage-grotesque,geist,jetbrains-mono}`.
  Bundled with the SPA — no CDN at runtime.

The shell talks to the host exclusively through public protocol:

- `POST /rpc` for all `kernel.v1.*` methods.
- `GET /kernel/v1/event.subscribe/:session_id` (SSE) for event tails.
- `postMessage` bridge for surfaces mounted in sandboxed iframes.

There is no SQLite access and no private runtime call. Shell-owned features that
call platform utility packages still go through ordinary `kernel.v1.capability.invoke`
paths; no official package receives a privileged side channel.

---

## Layout

```
src/
├── app.tsx                     # Provider tree (theme, kernel, tooltip, toast, icons)
├── main.tsx                    # createRoot entry point + font + CSS imports
├── styles/app.css              # Tailwind v4 @theme tokens, base, custom utilities
├── lib/
│   ├── theme.tsx               # Theme provider (system/light/dark, data-theme attr)
│   ├── router.ts               # Hash router — home / settings / project
│   ├── kernel-client.tsx       # KernelProvider, useKernel, useAsync, useEventTail
│   ├── format.ts               # Shared display helpers (relative time, bytes, etc)
│   ├── home-data.ts            # Mock fallbacks used when host unavailable
│   └── cn.ts                   # clsx + tailwind-merge composer
├── components/
│   ├── icons.tsx               # Phosphor re-exports with semantic names
│   ├── layout/
│   │   ├── shell.tsx           # Top-level <Shell />
│   │   ├── platform-topbar.tsx # 60px sticky topbar
│   │   └── settings-nav-rail.tsx
│   ├── ui/                     # Reusable primitives
│   │   ├── button.tsx          # CVA variants (primary/secondary/tertiary/destructive/icon)
│   │   ├── card.tsx            # Card, CardSection, CardRow
│   │   ├── modal.tsx           # Radix Dialog wrapper with motion + accent stripe
│   │   ├── dropdown.tsx        # Radix DropdownMenu wrapper
│   │   ├── tooltip.tsx         # Radix Tooltip wrapper
│   │   ├── toast.tsx           # In-house toast queue + viewport
│   │   ├── input.tsx           # Field, Input, InputGroup, Textarea, Checkbox
│   │   ├── status-pill.tsx     # State pills (running/stopped/failed/...)
│   │   ├── skeleton.tsx        # Shimmer placeholder
│   │   ├── empty-state.tsx     # Empty/error placeholder with optional retry
│   │   └── typography.tsx      # Eyebrow, HeroTitle, PageTitle, CardTitle, Mono
│   ├── home/                   # Home-page composition
│   │   ├── hero.tsx
│   │   ├── activity-micro-card.tsx
│   │   ├── utility-strip.tsx
│   │   ├── project-card.tsx
│   │   ├── install-card.tsx
│   │   ├── activity-timeline.tsx
│   │   └── workshop-utilities.tsx
│   └── install/
│       ├── install-modal.tsx   # install-lab wizard + external project branch
│       └── failure-modal.tsx   # redacted failure diagnostics with deep-rust accent
├── routes/
│   ├── home.tsx
│   ├── project-frame.tsx       # iframe wrapper around mounted surface
│   └── settings/
│       ├── index.tsx           # Tab dispatcher
│       ├── api-connections.tsx # secret-store-lab wired
│       ├── installed-packages.tsx # kernel.v1.package.list wired
│       ├── profiles.tsx        # kernel.v1.host.diagnostics wired
│       ├── storage.tsx         # storage areas + event store kind wired
│       └── about.tsx
├── protocol/
│   └── client.ts               # YggProtocolClient — typed RPC + SSE wrappers
└── surfaces/
    ├── surface-host.ts         # iframe SurfaceHost contract
    └── bundle-resolver.ts      # kernel.v1.surface.resolve_bundle wrapper
```

---

## Routes (hash-based)

| Hash | View |
| ---- | ---- |
| `#/` | Home |
| `#/settings/api-connections` | Settings — secrets |
| `#/settings/installed-packages` | Settings — package inventory |
| `#/settings/profiles` | Settings — workshop profiles |
| `#/settings/storage` | Settings — data paths and backend |
| `#/settings/about` | Settings — version, license, links |
| `#/project/<id>` | Project frame with mounted surface |

Hash routing was chosen over React Router because:

- The shell has five static routes; nothing dynamic enough to warrant the
  framework.
- It survives reloads inside Tauri WebView with no server config.
- It composes naturally with the surface iframe (the surface owns its own
  internal navigation independent of the shell route).

---

## Theming

`ThemeProvider` writes `data-theme="light" | "dark"` on `<html>`. Three preferences:

- `system` (default) — follows `prefers-color-scheme`.
- `light` / `dark` — explicit override, persisted in `localStorage` under
  `yggdrasil:theme-preference`.

Tailwind's `dark:` modifier is bound to `[data-theme="dark"]` via
`@custom-variant` in `app.css`. Modal overlay uses a dedicated
`--color-overlay` token that doesn't flip with theme so the scrim stays dark
in both modes. Brass accent shifts to a brighter `aged-brass-glow` in dark
mode for legibility on bark backgrounds.

---

## Real data wiring

| Page | Source |
| ---- | ------ |
| Home — projects | `kernel.v1.project.list` + per-project `storage_summary` |
| Settings — API Connections | `official/secret-store-lab/{list,put,delete}_secret` + `health` |
| Settings — Installed Packages | `kernel.v1.package.list` + `kernel.v1.project.list` (project flag) |
| Settings — Profiles | `kernel.v1.host.diagnostics` (active profile, packages_loaded, allowlist) |
| Settings — Storage | storage-area summary + event store kind |
| Project Frame | `kernel.v1.project.get/start/stop` + `kernel.v1.surface.resolve_bundle` |
| Install Modal | `official/install-lab/{resolve_plan,detect_kind,execute_plan}` through `kernel.v1.capability.invoke` |
| Failure Modal | `kernel.v1.package.list/status/logs` redacted failure summaries |

All async views show a shimmer skeleton during load and an `EmptyState` with a
retry action when the call fails. Mutating actions (delete secret, stop
project, install) push toast feedback and re-query the underlying resource.

The shell never reads raw secret values. Provider keys move from the secret
store into outbound requests via host-injected `secret_ref` references; the UI
sees only names, scopes, and counts.

---

## Surface hosting

`src/surfaces/surface-host.ts` mounts third-party surface bundles in sandboxed
iframes using `/surface-frame.html`. Surface bundles are ESM modules with a
named export that is either callable as `(root, props) => void` or exposes
`{ mount(root, props) }`.

The iframe uses `sandbox="allow-scripts"`. Host access is opt-in through the
explicit postMessage RPC bridge (`callRpc` and `subscribeEvents`). See
[`../../docs/guides/SURFACE_HOSTING.md`](../../docs/guides/SURFACE_HOSTING.md)
for the full contract.

Surface stream subscription is supported through additive postMessage messages
(`stream.subscribe`, `stream.frame`, `stream.ended`, `stream.error`,
`stream.unsubscribe`) bridged from host `kernel/v1/stream.*` events. YdlTavern
uses this for live model token streaming.

### ST URL layout (for SillyTavern extension compatibility)

YdlTavern surfaces serve SillyTavern-compatible ESM modules at standard ST URLs:

- `/script.js` — ST core globals shim
- `/scripts/extensions.js` — Extension manager shim
- `/scripts/events.js`, `/scripts/st-context.js`, `/scripts/group-chats.js`,
  `/scripts/secrets.js`, `/scripts/power-user.js`

These are served by the `ydltavern-st-compat-server` Vite plugin during dev,
reading from `../../YdlTavern/packages/ydltavern-surface/dist/st-compat/`.
Production hosting still needs a static fileserver route (deferred).

---

## Keyboard shortcuts

| Shortcut | Action |
| -------- | ------ |
| `⌘ N` / `Ctrl N` | Open Install modal (Home only) |
| `⌘ F` / `Ctrl F` | Focus package filter input (Settings → Installed Packages) |
| `Esc` | Close modal |
| `↵` | Confirm primary action in modals |

---

## Accessibility

- `:focus-visible` draws a 2px Aged Brass outline with offset on every
  focusable. Buttons use an inner ring; modal close uses the global ring.
- All interactive elements have `aria-label` when icon-only.
- Status pills include leading text in addition to color-coded dots.
- Toast queue uses `role="status"` + `aria-live="polite"`.
- `prefers-reduced-motion: reduce` zeroes animation/transition durations.

---

## What this shell is not

- It is not a Studio. There are no privileged tools that bypass public protocol.
- It is not a chat UI. Project surfaces own conversational behavior.
- It is not a marketplace. Settings → Installed Packages shows local
  inventory only; the web install flow accepts public HTTPS Git URLs, never a
  curated catalog.
- It is not a content runtime. All experience-level state lives in projects.

---

## Related docs

- [`../../docs/design/PLATFORM_UI_DESIGN.md`](../../docs/design/PLATFORM_UI_DESIGN.md)
  — Editorial Workshop design system reference.
- [`../../docs/guides/SURFACE_HOSTING.md`](../../docs/guides/SURFACE_HOSTING.md)
  — Surface bundle contract and mount lifecycle.
- [`../../docs/guides/PROJECT_MODEL.md`](../../docs/guides/PROJECT_MODEL.md)
  — Project lifecycle and Home card semantics.
- [`../../docs/guides/SECRET_MANAGEMENT.md`](../../docs/guides/SECRET_MANAGEMENT.md)
  — `secret_ref` contract and platform/project scoping.
- [`../../docs/spec/KERNEL_V1_CONTRACT.md`](../../docs/spec/KERNEL_V1_CONTRACT.md)
  — Public protocol that the shell consumes.
