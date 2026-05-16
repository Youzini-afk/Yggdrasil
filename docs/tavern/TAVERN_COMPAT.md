# Tavern Compatibility Plan

Tavern compatibility is implemented as a built-in runtime profile, not as the Yggdrasil kernel.

## Compatibility layers

### 1. Resource compatibility

Import and preserve community resources.

P0:

- Character Card V2,
- PNG metadata,
- World Info / Lorebook basics,
- first message,
- basic chat history later,
- prompt preset basics later.

P1:

- group chat,
- user persona,
- author's note,
- advanced world info options,
- regex/replacement basics,
- generation settings.

P2:

- extension-specific metadata,
- UI presets,
- third-party variants,
- stronger export compatibility.

### 2. Behavior compatibility

Make imported resources feel familiar enough to play.

Priority behavior:

- `{{char}}` / `{{user}}` replacement,
- description/personality/scenario/first_mes/mes_example,
- lorebook key matching,
- insertion order and depth basics,
- author note/system prompt concepts,
- stop strings,
- regenerate/edit/delete basics.

Yggdrasil should aim for compatible-enough behavior, not bug-for-bug compatibility.

### 3. Extension/shim compatibility

Extensions should be categorized:

1. Capability-like extensions: migrate to Yggdrasil capabilities.
2. UI-only extensions: later Studio contribution points.
3. Deep ST-internal extensions: migration guide or selective shim only.

## Import principle

Use lossless storage plus native projection:

```text
original_payload: original SillyTavern data
native_projection: Yggdrasil-native Asset/Actor/Memory/PromptProfile view
```

This preserves old resources without letting old schemas define the platform ceiling.
