# Documentation style and red lines

> [English](./STYLE.en.md) · [中文](./STYLE.md)

This document is the minimum set of rules for writing docs in the Yggdrasil repository. The goal: keep docs written for readers, not polluted by development logs.

## Write for readers, not for development logs

The primary audience for these docs is new engineers and external users. They want to know **what something is, how to use it, and where the boundary is**. They don't care which iteration shipped which line.

Do write:

- What the platform is, how the kernel is defined, what a capability package is, what a project is, what a surface is.
- How to run it, how to write a capability package, how to host a surface, how to manage secrets.
- Where the boundaries are: what is not in kernel scope, what lives in capability packages, what lives in projects.

Don't write:

- "We recently shipped X", "Round 10A.4 completed Y", "Phase B optimization moved Z to …".
- Doc sections that read like commit-message dumps.
- Already-completed work described as "in progress".

## No phase numbers

The repo no longer uses names like `Round X` / `Phase Y` / `Alpha Z` / `Beta N` / `T-track` / `U-track` in docs. They are temporary internal grouping labels and carry no meaning for readers.

Use these instead:

- Capabilities that exist in the repo → state the fact directly: "The kernel exposes X", "the package provides Y", "the surface uses Z".
- Work that is not done yet → `planned`, `deferred`, `still to be done`, `future work`.
- Done but still being polished → `partial` / `partial-real` / `partial-opt-in`, paired with a concrete delta description.

Do not number "the current state", and do not keep stale phase markers around.

## Status docs vs concept docs

Two kinds of docs, two narrative styles:

- **Concept docs** (`CHARTER`, `VISION`, `ARCHITECTURE`, `PLATFORM_KERNEL`, `CAPABILITY_PACKAGE`, `PLAY_CREATION_MODEL`, `KERNEL_V1_CONTRACT`, the guides) describe invariants, mechanisms, contracts, and usage. They should not be polluted by time — unless the platform stance or mechanism itself changes, leave them stable.
- **Status docs** (`ALPHA_STATUS`, `roadmap/NEXT_STEPS`, `spec/CONFORMANCE_MATRIX`, `COMPATIBILITY_MATRIX`) describe current state, partial, deferred, what's next. They are living documents and may include numbers, tables, and implementation progress — but they still should not read like a development log.

If a PR turns a concept doc into "we recently added phase X", that's the wrong direction — make X land in a status doc, and rewrite the relevant concept-doc paragraphs into stable description.

## ZH/EN 1:1 alignment

The main narrative, navigation, and primary guides must be maintained in both Chinese and English:

- File names: Chinese is `xxx.md` (default), English is `xxx.en.md`.
- The second line of each file is a bilingual blockquote: `> [English](./xxx.en.md) · [中文](./xxx.md)` for switching languages.
- Editing one side requires editing the other in the same change; drift is not allowed.
- Exceptions: `inventory/*.raw.md` machine-read scans are dominated by literal ST source identifiers and have no Chinese mirror; npm/cargo-style package / SDK READMEs are English-only by ecosystem convention.

## Doc red lines

Don't do these when writing docs:

- ❌ Naming docs like `ROUND_X_PLAN.md` / `PHASE_Y_DESIGN.md` / `*_ALPHA.md`. Temporary plan docs must be deleted as soon as the work is done, and durable content folded into README / the relevant guide / status docs.
- ❌ Pasting raw stderr / raw API keys / raw secrets into docs. When examples are needed, use `secret_ref:env:NAME`-style references.
- ❌ Writing host absolute paths (e.g. `/home/<user>/...`) into reader-facing guides. `~/.yggdrasil/<area>/` is fine, but don't expose machine specifics.
- ❌ Claiming "full-domain byte-level ST alignment" or "the kernel is SillyTavern-compatible" without fixtures and alignment tests to back it up.
- ❌ Stacking "Round X / Round X+1 / ..." completion lists in the main narrative. Completed phases belong to git history.
- ❌ Leaking integration-project semantics (chat, character, tavern, prompt, etc.) into Yggdrasil platform / kernel docs.

## Lifecycle of temporary plan docs

Plan docs (e.g. for a refactor, merge, or scrub) may live under `docs/roadmap/`, provided they:

- Carry a clear "this is a temporary plan and will be deleted when done" note at the top.
- Are deleted immediately when the work is finished — no stale roadmaps left in the repo.
- Have any long-term content (final architecture decisions, stable boundaries) folded into architecture / spec / guide / status docs rather than left in the plan doc.

## Ask before writing

Before adding or making major changes to a doc, ask:

1. Who is the reader? What do they need to understand?
2. Is this a concept doc or a status doc?
3. Is it redundant? Is the same fact already in README, ALPHA_STATUS, a guide, or somewhere else?
4. Will it leak phase numbers, development increments, stale roadmaps, host paths, or raw secrets?
5. Are the Chinese and English versions in sync?

Answering these tends to make the doc shorter and more stable.

## One-liner

**Docs are stable references for readers, not archives of the development process.**
