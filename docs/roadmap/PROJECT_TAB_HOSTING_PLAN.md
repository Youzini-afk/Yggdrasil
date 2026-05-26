# Project tab hosting plan

> Temporary execution plan. Delete this file after the implementation is merged into durable docs.

## Goal

Opening a project from Home should feel like opening that project as its own web app:

- Home remains open.
- The project opens in a separate tab at `/project/<project_id>`.
- The project tab has no Yggdrasil topbar, back button, or platform chrome.
- The surface still runs inside the existing sandboxed iframe and bridge-token model.
- Closing the tab does not stop the project session.
- `Cmd+.` / `Ctrl+.` stops the current project from the project tab.

## Non-goals

- Do not mount third-party/project surfaces directly into the host DOM.
- Do not give surfaces localStorage, kernel token, or same-origin access.
- Do not add a Ygg topbar or floating back button to the project tab.
- Do not stop sessions on `beforeunload`.
- Do not introduce a separate YdlTavern web server or independent app shell.

## Phase 0 — Plan and route/security review

- Record the chosen product behavior in this temporary plan.
- Keep the surface security model: `sandbox="allow-scripts"`, per-mount `bridge_token`, source checks, capability allowlists.
- Fix small bridge hardening gaps found during review before expanding the route.

## Phase 1 — Path route and chrome-free project host

- Teach the client router to parse `/project/<id>` from `window.location.pathname` as the project-host route.
- Keep existing hash routes for Home and Settings.
- Render `<ProjectFrame chrome="none" />` from the app root when path route is active, bypassing `<Shell>` and `<PlatformTopbar>`.
- Validate project ids as a single bounded URL segment.
- Set the document title to `<project title> — Yggdrasil` while mounted and restore it on unmount.
- Preserve existing auth behavior; project tabs reuse same-origin localStorage access tokens but never pass tokens into the iframe.

## Phase 2 — Home launch behavior

- Add a small launcher helper that opens `/project/<id>` in a sanitized named tab using `noopener,noreferrer`.
- Hash the project id for the target name instead of using the raw id.
- Use the launcher for ProjectCard, ContinueCard, and restart/open paths.
- Keep Home state and recent-project tracking unchanged.

## Phase 3 — Stop shortcut and no-chrome fallback states

- Add project-page-only `Cmd+.` / `Ctrl+.` stop handling.
- Require `event.isTrusted`, debounce while stopping, and ignore input/contenteditable targets.
- Do not expose a surface bridge method for stopping the project.
- Show loading and failure states inside the chrome-free project page.
- Keep no topbar and no persistent return control.

## Phase 4 — Tests and docs convergence

- Add router/launcher tests for path route validation, target sanitization, and open features.
- Add bootstrap/security tests for parent-source mount checks and wrong-token RPC result handling.
- Update `clients/web/README.md` and surface hosting docs with the separate-tab project host model.
- Run Web check/test/build, service tests for static fallback, and diff checks.
- Delete this plan after durable docs are updated.
