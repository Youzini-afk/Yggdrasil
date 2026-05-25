import { motion } from "motion/react";
import { Plus } from "@/components/icons";
import { SPRING } from "@/lib/motion";
import { cn } from "@/lib/cn";

export function InstallCard({
  onClick,
  index = 0,
  title = "Install a project",
  hint = "Paste a GitHub URL or local path",
}: {
  onClick: () => void;
  index?: number;
  title?: string;
  hint?: string;
}) {
  return (
    <motion.button
      type="button"
      onClick={onClick}
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: Math.min(index, 11) * 0.06, ...SPRING.soft }}
      whileHover={{ y: -2 }}
      className={cn(
        "flex flex-col items-center justify-center gap-3 rounded-[20px] border border-dashed border-whisper-border-strong/70 bg-transparent p-8 text-center transition hover:border-aged-brass-border hover:bg-aged-brass-surface-soft",
      )}
    >
      <span className="rounded-full border border-whisper-border bg-pure-surface p-2 text-steel-secondary">
        <Plus size={20} />
      </span>
      <div className="space-y-1">
        <p className="font-display text-[16px] font-bold text-charcoal-ink">{title}</p>
        <p className="text-[12px] text-muted-tone">{hint}</p>
      </div>
      <span className="font-mono text-[10px] text-muted-tone">⌘ N</span>
    </motion.button>
  );
}
