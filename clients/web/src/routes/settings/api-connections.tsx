import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { motion } from "motion/react";
import { Copy, DotsThree, Eye, EyeSlash, Key, Plus } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Card, CardSection } from "@/components/ui/card";
import { Eyebrow, EyebrowSm, PageTitle } from "@/components/ui/typography";
import { StatusPill } from "@/components/ui/status-pill";
import { Tooltip } from "@/components/ui/tooltip";
import { Field, Input } from "@/components/ui/input";
import { Modal, ModalHeader, ModalFooter } from "@/components/ui/modal";
import {
  Dropdown,
  DropdownTrigger,
  DropdownMenu,
  DropdownItem,
  DropdownSeparator,
} from "@/components/ui/dropdown";
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/ui/empty-state";
import { useToast } from "@/components/ui/toast";
import { useAsync, useKernel } from "@/lib/kernel-client";
import { cn } from "@/lib/cn";

interface DraftSecret {
  name: string;
  value: string;
}

interface SecretView {
  name: string;
  scope: string;
  scopeLabel: string;
  provider: string;
}

const PROVIDERS: Array<{ id: string; label: string }> = [
  { id: "OpenAI", label: "OpenAI" },
  { id: "Anthropic", label: "Anthropic" },
  { id: "Google", label: "Google" },
  { id: "OpenRouter", label: "OpenRouter" },
  { id: "DeepSeek", label: "DeepSeek" },
  { id: "xAI", label: "xAI" },
  { id: "Fireworks", label: "Fireworks" },
  { id: "Custom", label: "Custom" },
];

const PROVIDER_HINTS: Record<string, string> = {
  OPENAI_API_KEY: "OpenAI",
  ANTHROPIC_API_KEY: "Anthropic",
  GEMINI_API_KEY: "Google",
  OPENROUTER_API_KEY: "OpenRouter",
  DEEPSEEK_API_KEY: "DeepSeek",
  XAI_API_KEY: "xAI",
  FIREWORKS_API_KEY: "Fireworks",
};

export function ApiConnectionsPanel() {
  const client = useKernel();
  const toast = useToast();

  const platform = useAsync(() => client.listSecrets(), [client]);
  const health = useAsync(() => client.secretsHealth().catch(() => null), [client]);
  const [showAdd, setShowAdd] = useState(false);

  const secrets = useMemo<SecretView[]>(() => {
    return (platform.data ?? []).map((name) => ({
      name,
      scope: "platform",
      scopeLabel: "PLATFORM",
      provider: PROVIDER_HINTS[name] ?? "Custom",
    }));
  }, [platform.data]);

  const handleDelete = async (name: string) => {
    try {
      await client.deleteSecret(name);
      platform.refresh();
      health.refresh();
      toast.push({ variant: "info", title: `Removed ${name}` });
    } catch (err) {
      toast.push({
        variant: "error",
        title: "Delete failed",
        body: "The secret could not be removed. Check the local host and try again.",
      });
    }
  };

  const handleCopy = (name: string) => {
    navigator.clipboard?.writeText(name);
    toast.push({ variant: "success", title: "Copied secret name", duration: 2000 });
  };

  const handleSave = async (entry: DraftSecret) => {
    if (!entry.name || !entry.value) return;
    try {
      await client.putSecret(entry.name, entry.value);
      platform.refresh();
      health.refresh();
      setShowAdd(false);
      toast.push({ variant: "success", title: `Stored ${entry.name}` });
    } catch (err) {
      toast.push({
        variant: "error",
        title: "Save failed",
        body: "The secret could not be stored. Check the local host and try again.",
      });
    }
  };

  return (
    <>
      <header className="mb-8">
        <Eyebrow>
          {platform.loading
            ? "API Connections · loading…"
            : `API Connections · ${secrets.length} keys stored`}
        </Eyebrow>
        <PageTitle className="mt-2">Local secret store</PageTitle>
        <p className="mt-2 max-w-[60ch] text-[13px] leading-relaxed text-steel-secondary">
          Keys stay on this machine, encrypted with your platform key. Yggdrasil never transmits raw
          keys — projects request them through audited capability calls.
        </p>
      </header>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[7fr_3fr]">
        <section className="flex flex-col gap-3">
          <div className="flex items-center justify-between">
            <Eyebrow>Stored secrets</Eyebrow>
            <Button tone="tertiary" size="sm" onClick={() => setShowAdd(true)}>
              <Plus size={14} />
              Add secret
            </Button>
          </div>
          <Card>
            {platform.error ? (
              <EmptyState
                icon={<Key />}
                title="Couldn't load secrets"
                body="Secret metadata is unavailable. Try again from the local UI."
                action={{ label: "Retry", onClick: () => platform.refresh() }}
              />
            ) : platform.loading ? (
              <ul className="divide-y divide-whisper-border">
                {Array.from({ length: 3 }).map((_, idx) => (
                  <li key={idx} className="flex items-center gap-4 px-5 py-4">
                    <Skeleton className="size-9 rounded-full" />
                    <div className="flex-1 space-y-1.5">
                      <Skeleton className="h-3 w-40" />
                      <Skeleton className="h-2.5 w-60" />
                    </div>
                    <Skeleton className="h-7 w-7 rounded" />
                    <Skeleton className="h-7 w-7 rounded" />
                  </li>
                ))}
              </ul>
            ) : secrets.length === 0 ? (
              <EmptyState
                icon={<Key />}
                title="No secrets stored"
                body="Add your first key. Yggdrasil encrypts it with your platform key."
                action={{ label: "Add secret", onClick: () => setShowAdd(true) }}
              />
            ) : (
              <ul className="divide-y divide-whisper-border">
                {secrets.map((secret) => (
                  <SecretRow
                    key={secret.name}
                    secret={secret}
                    onCopy={() => handleCopy(secret.name)}
                    onDelete={() => handleDelete(secret.name)}
                  />
                ))}
              </ul>
            )}
          </Card>
        </section>

        <aside className="flex flex-col gap-4">
          <Card>
            <CardSection>
              <EyebrowSm>Store status</EyebrowSm>
              <dl className="mt-3 space-y-2 text-[12px]">
                {[
                  ["Encryption", "age (X25519)"],
                  ["Master key", health.data?.key_source ?? "—"],
                  ["Storage", health.data?.exists ? "configured" : "not created"],
                  [
                    "Total",
                    health.data
                      ? `${health.data.secret_count} secrets`
                      : platform.loading
                        ? "—"
                        : `${secrets.length} secrets`,
                  ],
                ].map(([label, value]) => (
                  <div key={label} className="flex items-center justify-between gap-3">
                    <dt className="text-steel-secondary">{label}</dt>
                    <dd className="truncate font-mono text-charcoal-ink">{value}</dd>
                  </div>
                ))}
              </dl>
            </CardSection>
          </Card>
          <Card>
            <CardSection>
              <EyebrowSm>How they're used</EyebrowSm>
              <p className="mt-3 text-[12px] leading-relaxed text-steel-secondary">
                Projects reference keys with{" "}
                <span className="font-mono text-charcoal-ink">secret_ref:store:NAME</span>. The host
                injects the raw value into outbound requests on the project's behalf.
              </p>
              <button
                type="button"
                className="mt-3 text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
              >
                Open audit log →
              </button>
            </CardSection>
          </Card>
          <Card>
            <CardSection>
              <EyebrowSm>Backup</EyebrowSm>
              <div className="mt-3 flex flex-col gap-1.5">
                <Button
                  tone="secondary"
                  size="sm"
                  className="justify-start"
                  onClick={() =>
                    toast.push({ variant: "info", title: "Use yg secrets export on the CLI" })
                  }
                >
                  Export to file
                </Button>
                <Button
                  tone="secondary"
                  size="sm"
                  className="justify-start"
                  onClick={() =>
                    toast.push({ variant: "info", title: "Use yg secrets import on the CLI" })
                  }
                >
                  Import from file
                </Button>
              </div>
            </CardSection>
          </Card>
        </aside>
      </div>

      <AddSecretModal open={showAdd} onClose={() => setShowAdd(false)} onSave={handleSave} />
    </>
  );
}

function SecretRow({
  secret,
  onCopy,
  onDelete,
}: {
  secret: SecretView;
  onCopy: () => void;
  onDelete: () => void;
}) {
  const [revealed, setRevealed] = useState(false);
  return (
    <motion.li layout className="flex items-center gap-4 px-5 py-4">
      <span className="rounded-full border border-whisper-border bg-aged-brass-surface-soft p-2 text-aged-brass">
        <Key size={16} />
      </span>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="truncate font-mono text-[13px] text-charcoal-ink">
            {revealed ? secret.name : "••••••••••••••"}
          </span>
          <StatusPill
            tone={secret.scope === "platform" ? "accent" : "neutral"}
            label={secret.scopeLabel}
            showDot={false}
          />
        </div>
        <p className="mt-1 text-[11px] text-steel-secondary">
          <span>{secret.provider}</span>
          <span className="mx-1.5 text-whisper-border-strong/70">·</span>
          <span className="font-mono text-muted-tone">secret_ref:store:{secret.name}</span>
        </p>
      </div>
      <Tooltip label={revealed ? "Hide name" : "Reveal name"}>
        <Button
          tone="icon"
          size="icon-sm"
          onClick={() => setRevealed((v) => !v)}
          aria-label="Toggle reveal"
        >
          {revealed ? <EyeSlash size={14} /> : <Eye size={14} />}
        </Button>
      </Tooltip>
      <Tooltip label="Copy name">
        <Button tone="icon" size="icon-sm" onClick={onCopy} aria-label="Copy">
          <Copy size={14} />
        </Button>
      </Tooltip>
      <Dropdown>
        <DropdownTrigger asChild>
          <Button tone="icon" size="icon-sm" aria-label="More">
            <DotsThree size={16} />
          </Button>
        </DropdownTrigger>
        <DropdownMenu>
          <DropdownItem>Rotate</DropdownItem>
          <DropdownSeparator />
          <DropdownItem destructive onSelect={onDelete}>
            Delete…
          </DropdownItem>
        </DropdownMenu>
      </Dropdown>
    </motion.li>
  );
}

function AddSecretModal({
  open,
  onClose,
  onSave,
}: {
  open: boolean;
  onClose: () => void;
  onSave: (entry: DraftSecret) => void;
}) {
  const [name, setName] = useState("");
  const [provider, setProvider] = useState("OpenAI");
  const [scope, setScope] = useState<string>("platform");
  const valueRef = useRef<HTMLInputElement>(null);

  const wipeValueInput = () => {
    if (valueRef.current) valueRef.current.value = "";
  };

  // Always wipe the raw secret from React memory when the modal closes —
  // success path, cancel path, esc path, anywhere. The host has the
  // encrypted copy; the UI must never retain raw values past submission.
  useEffect(() => {
    if (!open) {
      wipeValueInput();
      setName("");
      setProvider("OpenAI");
      setScope("platform");
    }
  }, [open]);

  const handleSave = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    const value = String(data.get("secret-value") ?? "");
    if (!name || !value) return;
    onSave({ name, value });
    // Wipe immediately on submit; do not wait for the close effect.
    wipeValueInput();
    setName("");
  };

  return (
    <Modal open={open} onOpenChange={onClose} contentLabel="Add secret">
      <ModalHeader
        eyebrow="API Connections · Add"
        title="Store a new key"
        description="Yggdrasil encrypts the value with your platform key and never sends raw keys to any project."
      />
      <form onSubmit={handleSave} className="flex flex-col gap-4">
        <Field label="Provider" required>
          <select
            value={provider}
            onChange={(e) => setProvider(e.target.value)}
            className="h-10 rounded-[10px] border border-whisper-border bg-transparent px-3 text-[13px] outline-none transition focus-visible:border-aged-brass focus-visible:ring-2 focus-visible:ring-aged-brass/40"
          >
            {PROVIDERS.map((p) => (
              <option key={p.id}>{p.label}</option>
            ))}
          </select>
        </Field>
        <Field
          label="Secret name"
          helper="Convention: PROVIDER_API_KEY (uppercase, underscores)"
          required
        >
          <Input
            value={name}
            onChange={(e) => setName(e.target.value.toUpperCase())}
            placeholder="OPENAI_API_KEY"
            spellCheck={false}
          />
        </Field>
        <Field label="Value" helper="The raw key never leaves this machine." required>
          <Input
            type="password"
            ref={valueRef}
            name="secret-value"
            placeholder="sk-…"
            autoComplete="new-password"
          />
        </Field>
        <Field label="Scope">
          <div className="flex flex-wrap gap-2">
            {[
              { id: "platform", label: "Platform-wide" },
              { id: "project", label: "Project-only (configure on Home)" },
            ].map((option) => (
              <button
                key={option.id}
                type="button"
                onClick={() => setScope(option.id)}
                disabled={option.id !== "platform"}
                className={cn(
                  "rounded-full border px-3 py-1 text-[12px] font-medium transition disabled:opacity-50",
                  scope === option.id
                    ? "border-aged-brass-border bg-aged-brass-surface text-charcoal-ink"
                    : "border-whisper-border text-steel-secondary hover:bg-whisper-border-strong/30",
                )}
              >
                {option.label}
              </button>
            ))}
          </div>
        </Field>
        <ModalFooter className="justify-end">
          <Button tone="secondary" type="button" onClick={onClose}>
            Cancel
          </Button>
          <Button tone="primary" type="submit" disabled={!name}>
            Save key
          </Button>
        </ModalFooter>
      </form>
    </Modal>
  );
}
