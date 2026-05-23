# Platform UI Scope and User Journeys

> [English](./PLATFORM_UI_SCOPE.en.md) · [中文](./PLATFORM_UI_SCOPE.md)

This document lists the UI scope of the Yggdrasil platform shell
(`clients/web`), the user journeys connecting them, the component catalog, and
the prioritized design surface list. Visual rules live in
`PLATFORM_UI_DESIGN.md`.

---

## Boundary

**Do** — platform shell UI:
- Home project shelf
- Settings panels (API Connections / Installed Packages / Profiles / About)
- Install flow (URL input, native vs external detection, wizard, progress)
- Project frame (chrome above mounted iframe)
- Notification / Toast system
- Empty states / Error states
- Failure details / retry entry
- Cmd+K command palette (deferred)

**Don't** — project internal UI:
- YdlTavern's own UI (preserved as ST-fork for community extension compat)
- Any project's entry surface internals
- The seam between project surface and platform is handled by the
  surface-host iframe boundary; projects are free inside

---

## User Journeys (by importance)

### J1: First run (clean install)

```
Launch Yggdrasil desktop
  → Platform shell loads, host starts
  → Home route shows empty shelf
  → First-run guidance:
      Core message: "Your workshop is empty. Install a project to begin."
      Single action: large "+ Install your first project" card
      (Optional) one-liner mentioning yg install CLI also works
```

### J2: Install YdlTavern

```
Click "+ Install your first project"
  → Install Modal opens
  → URL input field (placeholder: github.com/user/repo or ./local/path)
  → User pastes github.com/Youzini-afk/Yggdrasil-Tavern, presses Enter
  → Modal switches to "Resolving" state: shows git probe progress
  → install-lab.detect_kind returns native (project.yaml present)
  → Modal switches to "Confirm plan"
      Project: YdlTavern (signed: false)
      Dependencies: 2 packages (official + this repo)
      Permissions requested:
        - network: api.openai.com, api.anthropic.com, ...
        - secrets: secret_ref:store:OPENAI_API_KEY etc.
      Signature: ⚠ unsigned (allowed)
      Conformance: ✓ pass
      [Cancel] [Install]
  → User clicks Install
  → Modal switches to "Installing": progress bar + current step
      Fetch source → Verify integrity → Write store → Register project → Done
  → Install complete: modal auto-closes
  → Toast: "YdlTavern installed" + jump-to-Home action
  → Home shelf now has one card: YdlTavern (Stopped)
```

### J3: Install external project (no project.yaml)

```
Click "+ Install project"
  → Install Modal, URL input
  → install-lab.detect_kind returns external
  → Modal switches to wizard:
      "This repository doesn't declare itself as a Yggdrasil project. How
       do you want to use it?"
      [Option A] Wrap with adapter — Yggdrasil generates an adapter package
                 (recommended when source is a tool)
      [Option B] Open as workspace — agent-assisted, no wrapping
                 (recommended when you want to explore)
      [Cancel]
  → User picks B
  → Subsequent steps same as J2 (Confirm → Install → Done)
  → Home shelf shows card with type: external_workspace
```

### J4: Play a project

```
Home shelf → hover YdlTavern card (subtle lift + shadow deepens)
  → Click "▶ Play"
  → Card state: Stopped → Starting (yellow shimmer)
  → Background: kernel.v1.project.start → state machine → opens session
  → Card state: Starting → Running (Aged Brass + pulse)
  → Route transitions to Project frame
  → 40px project topbar appears above iframe:
      [← Back] [icon] YdlTavern  [• Running]    [Stop] [⋯]
  → Below the topbar, the entire viewport is given to the project iframe
  → YdlTavern's own UI renders (ST DOM fork)
  → User starts using the project
```

### J5: Configure API key

Two entry points:

**Entry A — inside project** (YdlTavern already implements this; not redesigning):
```
Project's API Connections drawer → paste → save
  → Choose scope: Platform-wide / This project only
  → secret-store-lab.put_secret or put_project_secret
```

**Entry B — platform Settings** (new, platform shell needs to build):
```
Topbar gear icon → Settings route
  → Left nav: API Connections / Installed Packages / Profiles / About
  → Pick "API Connections" panel
  → Shows saved secrets list (name, not value, created time, related project count)
  → "+ Add new secret" button
  → Small dialog: provider picker / name / value / scope
      Scope: Platform / selected project (if a project is running)
  → Save → secret-store-lab.put_secret
  → List refreshes
  → User can edit, delete entries
  → Delete confirms via dialog
```

### J6: Manage installed packages

```
Settings → Installed Packages
  → Installed projects and packages list
      Each row: package ID (mono) | version | source (git/local/internal) |
                state | size on disk | last updated
  → Top filter: All / Projects only / Dependencies only / by state
  → Inline actions: ⋯ menu (Update / Uninstall / View permissions / View logs)
  → Click Update: check upstream → show changelog → confirm
  → Click Uninstall:
      If a project, opens keep-data / delete-data choice (per Round 10A.2)
      If a dependency package, warns other projects may be affected
  → Click View permissions: shows manifest.permissions detail
```

### J7: Switch profile

```
Settings → Profiles
  → Shows active profile + other available profiles
  → Active: default (forge-alpha)
  → Switching requires host restart
  → Shows "Switching restarts: [Cancel] [Restart host into alpha]"
  → Restart shows full-screen loading then returns to Home (under new profile)
```

### J8: Project crash / failure

```
Project running, suddenly Failed (subprocess crash / timeout / error event)
  → Card state turns Deep Rust
  → Toast slides in: "YdlTavern stopped (subprocess crash). [View details]"
  → User clicks details → opens Failure modal
      Shows: exit code / stderr last 50 lines (mono) / timestamp
      Actions: [Copy log] [Restart project] [Close]
  → User clicks Restart → kernel.v1.project.start (transparent restart)
```

### J9: Command palette (Cmd+K, deferred)

```
Any route, Cmd+K (or Ctrl+K)
  → Search modal floats center, backdrop blurs
  → Input + live search results list
  → Search scope: project names, settings entries, commands (Install, Switch profile)
  → Arrow keys select, Enter executes, Esc dismisses
  → Linear / Raycast style
```

### J10: Theme toggle

```
Topbar right sun/moon icon → toggle light/dark
  → Instant (CSS variables swap)
  → localStorage persists, default reads prefers-color-scheme
```

---

## Surfaces (by design priority)

### Batch 1 — platform core (sets tone, must come first)

1. **Home (light + dark)** — project shelf, asymmetric hero, editorial workshop
2. **Home Empty** — first-run state with no projects
3. **Install Modal — URL input** — flow entry
4. **Install Modal — Plan confirm** — packages list, permissions, signature
5. **Install Modal — Progress** — installing
6. **External Project Wizard** — wrap / workspace choice
7. **Project Frame topbar** — chrome above mounted iframe
8. **Toast / Notification** — toast styles for various states

### Batch 2 — Settings trio

9. **Settings — API Connections** — platform secret management
10. **Settings — Installed Packages** — package management
11. **Settings — Profiles** — profile switching

### Batch 3 — Recovery & polish

12. **Failure Modal** — project crash details
13. **Loading / Skeleton states** — skeleton patterns
14. **Settings — About** — platform version, license, links

### Batch 4 — deferred (only if time)

15. **Cmd+K Command Palette**
16. **Project detail (deep view)**

---

## Component catalog (reusable)

By frequency, high to low:

- **Project card** (Home main element)
- **Status pill** (Running / Stopped / Starting / Failed / Updating)
- **Primary button / Secondary / Destructive / Icon button**
- **Form input** (text / search / password / select / radio)
- **Toast** (info / success / error / warning, but warnings use Aged Brass not yellow)
- **Modal** (form / wizard / confirm)
- **Settings nav rail** (left secondary nav)
- **Settings row** (label + control + helper + divider)
- **Empty state** (composed icon + heading + body + optional CTA)
- **Error banner** (inline error container)
- **Skeleton loader** (card / row / panel variants)
- **Top bar** (platform + project frame variants)
- **Drop menu** (⋯ menu, Radix-based)
- **Tooltip** (hover info)
- **Tabs / Segmented control** (settings)
- **Progress bar** (install flow)

---

## Platform / project boundary

```
┌────────────────────────────────────────────┐
│  Platform Topbar (60px)                              │
│  Yggdrasil    /    Project: YdlTavern    [⚙] [🌗]    │
├────────────────────────────────────────────┤
│  Project Frame Topbar (40px)                         │
│  [←]  YdlTavern  • Running              [Stop] [⋯]  │
├────────────────────────────────────────────┤
│                                                       │
│           Project iframe (free territory)             │
│           YdlTavern's own DOM lives here              │
│                                                       │
└────────────────────────────────────────────┘
```

- Platform topbar always visible (sticky)
- Project frame topbar appears only when a project is mounted, between
  platform topbar and iframe
- iframe content (YdlTavern or any project) is fully free; platform does not
  bleed in
- Communication via existing postMessage RPC bridge (Round 10A series)

---

## Stitch generation order

Step 1 (this round): Home (light), Home (dark), Home Empty, Install Modal URL input
  → Review images, iterate visual style. Adjust DESIGN.md until satisfied.

Step 2: After consistency settled, batch-generate Install Plan / Progress /
Wizard / Project Topbar / Toast

Step 3: Settings trio

Step 4: Recovery + skeletons + deferred screens

Each step: @designer takes the screens to React implementation, no piling up
unimplemented Figma snapshots.

---

## Pending decisions

1. **Topbar single or double layer?**
   - Double (60+40): platform always present, project topbar overlays when
     mounted (recommended)
   - Single (60): platform topbar morphs when project mounts
   - Recommend: double, clear boundary

2. **First-run welcome screen?**
   - Yes: dedicated welcome page with logo + slogan + CTA before first install
   - No: direct to Home empty state, let it guide
   - Recommend: No (Home empty state can guide, avoid onboarding bloat)

3. **Marketing landing page?**
   - Public yggdrasil.dev style site? Or only the desktop shell?
   - Recommend: only desktop shell now; marketing page deferred

4. **Logo type?**
   - Cabinet Grotesk text logo (recommended, simple direct)
   - Future: simple mark
   - Recommend: text (already confirmed this round)
