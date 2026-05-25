import { Button } from "@/components/ui/button";
import {
  Dropdown,
  DropdownMenu,
  DropdownItem,
  DropdownTrigger,
} from "@/components/ui/dropdown";
import { Tooltip } from "@/components/ui/tooltip";
import { useLocale, type SupportedLocale } from "@/lib/locale";
import { cn } from "@/lib/cn";

const LANGS = [
  { code: "zh-CN", label: "简体中文" },
  { code: "en", label: "English" },
] satisfies Array<{ code: SupportedLocale; label: string }>;

export function LocaleSwitcher() {
  const { locale, setLocale, t } = useLocale();
  const active = LANGS.find((l) => l.code === locale) ?? LANGS[0];

  return (
    <Dropdown>
      <Tooltip label={t("languageMenuLabel")}>
        <DropdownTrigger asChild>
          <Button
            tone="icon"
            size="icon"
            aria-label={t("languageAria", active.label)}
            aria-haspopup="menu"
          >
            <span className="font-mono text-[11px] font-semibold uppercase leading-none">
              {t("languageShort")}
            </span>
          </Button>
        </DropdownTrigger>
      </Tooltip>
      <DropdownMenu align="end">
        {LANGS.map((l) => (
          <DropdownItem
            key={l.code}
            onSelect={() => setLocale(l.code)}
            className="justify-between"
          >
            <span className={cn(l.code === locale && "font-medium")}>
              {l.label}
            </span>
            {l.code === locale ? (
              <span className="text-aged-brass">●</span>
            ) : null}
          </DropdownItem>
        ))}
      </DropdownMenu>
    </Dropdown>
  );
}
