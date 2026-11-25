import {
  Command,
  FileSpreadsheet,
  Home,
  Moon,
  Settings,
  Sparkles,
  Sun,
} from "lucide-react";
import {
  QlevelMark,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
  cn,
} from "@qlevel/design-system";
import type { Dictionary } from "../i18n";

export type WorkspaceScreen = "intake" | "live" | "results";

export interface IconRailProps {
  activeScreen: WorkspaceScreen;
  hasResults: boolean;
  isBusy: boolean;
  dark: boolean;
  dict: Dictionary;
  onSelectScreen: (screen: WorkspaceScreen) => void;
  onOpenCommand: () => void;
  onToggleDark: () => void;
  onOpenSettings: () => void;
}

export function IconRail({
  activeScreen,
  hasResults,
  isBusy,
  dark,
  dict,
  onSelectScreen,
  onOpenCommand,
  onToggleDark,
  onOpenSettings,
}: IconRailProps) {
  const navItems: readonly {
    id: WorkspaceScreen;
    label: string;
    icon: typeof Home;
    disabled?: boolean;
  }[] = [
    { id: "intake", label: dict.navIntake, icon: Home },
    {
      id: "live",
      label: dict.navAgentStream,
      icon: Sparkles,
      disabled: !isBusy && activeScreen !== "live",
    },
    {
      id: "results",
      label: dict.navResults,
      icon: FileSpreadsheet,
      disabled: !hasResults,
    },
  ];

  return (
    <TooltipProvider delayDuration={120}>
      <aside className="flex w-12 shrink-0 flex-col items-center justify-between border-e border-[var(--border)] bg-[var(--secondary)]/55 py-3 select-none">
        <div className="flex flex-col items-center gap-2">
          <div className="mb-1">
            <QlevelMark size={26} />
          </div>

          {navItems.map((item) => {
            const Icon = item.icon;
            const active = activeScreen === item.id;
            return (
              <Tooltip key={item.id}>
                <TooltipTrigger asChild>
                  <button
                    type="button"
                    disabled={item.disabled}
                    aria-label={item.label}
                    onClick={() => onSelectScreen(item.id)}
                    className={cn(
                      "flex h-8 w-8 items-center justify-center rounded-lg transition-colors cursor-pointer disabled:opacity-30 disabled:pointer-events-none",
                      active
                        ? "bg-[var(--card)] text-[var(--foreground)] shadow-2xs border border-[var(--border)]"
                        : "text-[var(--muted-foreground)] hover:bg-[var(--card)]/70 hover:text-[var(--foreground)]"
                    )}
                  >
                    <Icon className="h-4 w-4" />
                  </button>
                </TooltipTrigger>
                <TooltipContent side="right">{item.label}</TooltipContent>
              </Tooltip>
            );
          })}

          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type="button"
                aria-label={dict.navCommand}
                onClick={onOpenCommand}
                className="flex h-8 w-8 items-center justify-center rounded-lg text-[var(--muted-foreground)] hover:bg-[var(--card)]/70 hover:text-[var(--foreground)] transition-colors cursor-pointer"
              >
                <Command className="h-4 w-4" />
              </button>
            </TooltipTrigger>
            <TooltipContent side="right">{dict.navCommand}</TooltipContent>
          </Tooltip>
        </div>

        <div className="flex flex-col items-center gap-2">
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type="button"
                aria-label={dict.navTheme}
                onClick={onToggleDark}
                className="flex h-8 w-8 items-center justify-center rounded-lg text-[var(--muted-foreground)] hover:bg-[var(--card)]/70 hover:text-[var(--foreground)] transition-colors cursor-pointer"
              >
                {dark ? (
                  <Sun className="h-4 w-4" />
                ) : (
                  <Moon className="h-4 w-4" />
                )}
              </button>
            </TooltipTrigger>
            <TooltipContent side="right">{dict.navTheme}</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type="button"
                aria-label={dict.navSettings}
                onClick={onOpenSettings}
                className="flex h-8 w-8 items-center justify-center rounded-lg text-[var(--muted-foreground)] hover:bg-[var(--card)]/70 hover:text-[var(--foreground)] transition-colors cursor-pointer"
              >
                <Settings className="h-4 w-4" />
              </button>
            </TooltipTrigger>
            <TooltipContent side="right">{dict.navSettings}</TooltipContent>
          </Tooltip>
        </div>
      </aside>
    </TooltipProvider>
  );
}
