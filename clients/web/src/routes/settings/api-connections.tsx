import { useState } from "react";
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
import { useToast } from "@/components/ui/toast";
import { cn } from "@/lib/cn";

interface SecretEntry {
  id: string;
  name: string;
  provider: string;
  scope: "platform" | string; // project id otherwise
  scopeLabel: string;
  usageCount: number;
  updatedAge: string;
}

const MOCK_SECRETS: SecretEntry[] = [
  {
    id: "s1",
    name: "OPENAI_API_KEY",
    provider: "OpenAI",
    scope: "platform",
    scopeLabel: "PLATFORM",
    usageCount: 2,
    updatedAge: "3 days ago",
  },
  {
    id: "s2",
    name: "ANTHROPIC_API_KEY",
    provider: "Anthropic",
    scope: "platform",
    scopeLabel: "PLATFORM",
    usageCount: 1,
    updatedAge: "1 week ago",
  },
  {
    id: "s3",
    name: "GEMINI_API_KEY",
    provider: "Google",
    scope: "platform",
    scopeLabel: "PLATFORM",
    usageCount: 0,
    updatedAge: "2 weeks ago",
  },
  {
    id: "s4",
    name: "OPENROUTER_API_KEY",
    provider: "OpenRouter",
    scope: "ydltavern",
    scopeLabel: "YDLTAVERN ONLY",
    usageCount: 1,
    updatedAge: "yesterday",
  },
  {
    id: "s5",
    name: "DEEPSEEK_API_KEY",
    provider: "DeepSeek",
    scope: "coding-workshop",
    scopeLabel: "CODING WORKSHOP ONLY",
    usageCount: 1,
    updatedAge: "4 days ago",
  },
];

export function ApiConnectionsPanel() {
  const toast = useToast();
  const [secrets, setSecrets] = useState(MOCK_SECRETS);
  const [showAdd, setShowAdd] = useState(false);

  const handleDelete = (id: string, name: string) => {
    setSecrets((current) => current.filter((s) => s.id !== id));
    toast.push({ variant: "info", title: `Removed ${name}` });
  };

  const handleCopy = (name: string) => {
    navigator.clipboard?.writeText(name);
    toast.push({ variant: "success", title: "Copied secret name" });
  };

  return (
    <>
      <header className="mb-8">
        <Eyebrow>API Connections · {secrets.length} keys stored</Eyebrow>
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
            {secrets.length === 0 ? (
              <CardSection className="text-center text-[13px] text-muted-tone">
                No secrets stored. Add your first key above.
              </CardSection>
            ) : (
              <ul className="divide-y divide-whisper-border">
                {secrets.map((secret) => (
                  <SecretRow
                    key={secret.id}
                    secret={secret}
                    onCopy={() => handleCopy(secret.name)}
                    onDelete={() => handleDelete(secret.id, secret.name)}
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
                  ["Master key", "OS keychain"],
                  ["Path", "~/.yggdrasil/secrets.dat"],
                  ["Total", `${secrets.length} secrets · 1.2 KB`],
                ].map(([label, value]) => (
                  <div key={label} className="flex items-center justify-between">
                    <dt className="text-steel-secondary">{label}</dt>
                    <dd className="font-mono text-charcoal-ink">{value}</dd>
                  </div>
                ))}
              </dl>
            </CardSection>
          </Card>
          <Card>
            <CardSection>
              <EyebrowSm>Access last 24h</EyebrowSm>
              <p className="mt-3 font-display text-[22px] font-bold leading-none text-charcoal-ink">
                47 requests
              </p>
              <ul className="mt-2 text-[11px] text-steel-secondary">
                <li>YdlTavern · 41</li>
                <li>Coding Workshop · 6</li>
              </ul>
              <button
                type="button"
                className="mt-3 text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
              >
                View audit log →
              </button>
            </CardSection>
          </Card>
          <Card>
            <CardSection>
              <EyebrowSm>Backup</EyebrowSm>
              <div className="mt-3 flex flex-col gap-1.5">
                <Button tone="secondary" size="sm" className="justify-start">
                  Export to file
                </Button>
                <Button tone="secondary" size="sm" className="justify-start">
                  Import from file
                </Button>
              </div>
            </CardSection>
          </Card>
        </aside>
      </div>

      <AddSecretModal
        open={showAdd}
        onClose={() => setShowAdd(false)}
        onSave={(entry) => {
          setSecrets((current) => [...current, { ...entry, id: crypto.randomUUID() }]);
          setShowAdd(false);
          toast.push({ variant: "success", title: `Added ${entry.name}` });
        }}
      />
    </>
  );
}

function SecretRow({
  secret,
  onCopy,
  onDelete,
}: {
  secret: SecretEntry;
  onCopy: () => void;
  onDelete: () => void;
}) {
  const [revealed, setRevealed] = useState(false);
  return (
    <motion.li
      layout
      className="flex items-center gap-4 px-5 py-4"
    >
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
          <span>
            used by {secret.usageCount} project{secret.usageCount === 1 ? "" : "s"}
          </span>
          <span className="mx-1.5 text-whisper-border-strong/70">·</span>
          <span className="font-mono text-muted-tone">Updated {secret.updatedAge}</span>
        </p>
      </div>
      <Tooltip label={revealed ? "Hide name" : "Reveal name"}>
        <Button tone="icon" size="icon-sm" onClick={() => setRevealed((v) => !v)} aria-label="Toggle reveal">
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
          <DropdownItem>Edit scope…</DropdownItem>
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
  onSave: (entry: Omit<SecretEntry, "id">) => void;
}) {
  const [name, setName] = useState("");
  const [provider, setProvider] = useState("OpenAI");
  const [scope, setScope] = useState<"platform" | string>("platform");

  const handleSave = () => {
    if (!name) return;
    onSave({
      name,
      provider,
      scope,
      scopeLabel: scope === "platform" ? "PLATFORM" : scope.toUpperCase(),
      usageCount: 0,
      updatedAge: "just now",
    });
    setName("");
  };

  return (
    <Modal open={open} onOpenChange={onClose} contentLabel="Add secret">
      <ModalHeader
        eyebrow="API Connections · Add"
        title="Store a new key"
        description="Yggdrasil encrypts the value with your platform key and never sends raw keys to any project."
      />
      <div className="flex flex-col gap-4">
        <Field label="Provider" required>
          <select
            value={provider}
            onChange={(e) => setProvider(e.target.value)}
            className={cn(
              "h-10 rounded-[10px] border border-whisper-border bg-transparent px-3 text-[13px] outline-none transition focus-visible:border-aged-brass focus-visible:ring-2 focus-visible:ring-aged-brass/40",
            )}
          >
            <option>OpenAI</option>
            <option>Anthropic</option>
            <option>Google</option>
            <option>OpenRouter</option>
            <option>DeepSeek</option>
            <option>xAI</option>
            <option>Fireworks</option>
            <option>Custom</option>
          </select>
        </Field>
        <Field label="Secret name" helper="Convention: PROVIDER_API_KEY (uppercase, underscores)" required>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value.toUpperCase())}
            placeholder="OPENAI_API_KEY"
            spellCheck={false}
          />
        </Field>
        <Field label="Value" helper="The raw key never leaves this machine.">
          <Input type="password" placeholder="sk-…" autoComplete="new-password" />
        </Field>
        <Field label="Scope">
          <div className="flex flex-wrap gap-2">
            {[
              { id: "platform", label: "Platform-wide" },
              { id: "ydltavern", label: "YdlTavern only" },
              { id: "coding-workshop", label: "Coding Workshop only" },
            ].map((option) => (
              <button
                key={option.id}
                type="button"
                onClick={() => setScope(option.id)}
                className={cn(
                  "rounded-full border px-3 py-1 text-[12px] font-medium transition",
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
      </div>
      <ModalFooter className="justify-end">
        <Button tone="secondary" onClick={onClose}>
          Cancel
        </Button>
        <Button tone="primary" onClick={handleSave} disabled={!name}>
          Save key
        </Button>
      </ModalFooter>
    </Modal>
  );
}
