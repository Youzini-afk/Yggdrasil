import { type ReactNode } from "react";
import * as RadixDropdown from "@radix-ui/react-dropdown-menu";
import { motion, AnimatePresence } from "motion/react";
import { cn } from "@/lib/cn";

export const Dropdown = RadixDropdown.Root;
export const DropdownTrigger = RadixDropdown.Trigger;

export interface DropdownMenuProps {
  children: ReactNode;
  align?: "start" | "center" | "end";
  className?: string;
  open?: boolean;
}

export function DropdownMenu({ children, align = "end", className }: DropdownMenuProps) {
  return (
    <RadixDropdown.Portal>
      <RadixDropdown.Content asChild align={align} sideOffset={6}>
        <motion.div
          initial={{ opacity: 0, y: -4, scale: 0.96 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, scale: 0.96 }}
          transition={{ type: "spring", stiffness: 520, damping: 38 }}
          className={cn(
            "z-50 min-w-[180px] rounded-[10px] border border-whisper-border bg-pure-surface p-1 shadow-toast",
            className,
          )}
        >
          {children}
        </motion.div>
      </RadixDropdown.Content>
    </RadixDropdown.Portal>
  );
}

export function DropdownItem({
  children,
  onSelect,
  destructive,
  className,
}: {
  children: ReactNode;
  onSelect?: () => void;
  destructive?: boolean;
  className?: string;
}) {
  return (
    <RadixDropdown.Item
      onSelect={onSelect}
      className={cn(
        "flex cursor-pointer select-none items-center gap-2 rounded-[6px] px-2.5 py-1.5 text-[12px] outline-none transition",
        "data-[highlighted]:bg-whisper-border-strong/40",
        destructive ? "text-deep-rust" : "text-charcoal-ink",
        className,
      )}
    >
      {children}
    </RadixDropdown.Item>
  );
}

export const DropdownSeparator = () => (
  <RadixDropdown.Separator className="my-1 h-px bg-whisper-border" />
);
