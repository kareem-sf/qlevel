import * as DialogPrimitive from "@radix-ui/react-dialog";
import * as ScrollAreaPrimitive from "@radix-ui/react-scroll-area";
import * as SelectPrimitive from "@radix-ui/react-select";
import { Slot } from "@radix-ui/react-slot";
import * as SwitchPrimitive from "@radix-ui/react-switch";
import * as TabsPrimitive from "@radix-ui/react-tabs";
import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import { cva, type VariantProps } from "class-variance-authority";
import { Command as CommandPrimitive } from "cmdk";
import { Check, ChevronDown, Search, X } from "lucide-react";
import type {
  ButtonHTMLAttributes,
  ComponentPropsWithoutRef,
  HTMLAttributes,
  InputHTMLAttributes,
  ReactNode,
  TextareaHTMLAttributes,
} from "react";
import { Toaster as SonnerToaster } from "sonner";
import { cn } from "./lib/utils";

export const buttonVariants = cva(
  "inline-flex items-center justify-center gap-1.5 whitespace-nowrap rounded-lg text-xs font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ring)] disabled:pointer-events-none disabled:opacity-45 cursor-pointer select-none",
  {
    variants: {
      variant: {
        primary:
          "bg-[var(--primary)] text-[var(--primary-foreground)] hover:opacity-90 shadow-2xs",
        secondary:
          "border border-[var(--border)] bg-[var(--card)] text-[var(--foreground)] hover:bg-[var(--secondary)]",
        subtle:
          "bg-[var(--secondary)] text-[var(--secondary-foreground)] hover:opacity-85",
        ghost:
          "text-[var(--muted-foreground)] hover:bg-[var(--secondary)] hover:text-[var(--foreground)]",
        danger:
          "bg-[var(--destructive)] text-white hover:opacity-90",
      },
      size: {
        sm: "h-7 px-2.5 text-[11px] rounded-md",
        md: "h-8 px-3.5 text-xs",
        lg: "h-9 px-4 text-[13px] font-semibold",
        icon: "h-8 w-8 p-0",
      },
    },
    defaultVariants: {
      variant: "primary",
      size: "md",
    },
  }
);

export interface ButtonProps
  extends ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean | undefined;
}

export function Button({
  className,
  variant,
  size,
  asChild = false,
  type = "button",
  ...props
}: ButtonProps) {
  const Comp = asChild ? Slot : "button";
  return (
    <Comp
      type={type}
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    />
  );
}

export const badgeVariants = cva(
  "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-[11px] font-medium transition-colors",
  {
    variants: {
      tone: {
        neutral:
          "border-[var(--border)] bg-[var(--secondary)] text-[var(--secondary-foreground)]",
        outline:
          "border-[var(--border)] bg-[var(--card)] text-[var(--muted-foreground)]",
        success:
          "border-emerald-500/25 bg-[var(--emerald-subtle)] text-[var(--emerald-fg)]",
        warning:
          "border-amber-500/25 bg-[var(--amber-subtle)] text-[var(--amber-fg)]",
        danger:
          "border-red-500/25 bg-[var(--destructive-subtle)] text-[var(--destructive-fg)]",
      },
    },
    defaultVariants: {
      tone: "neutral",
    },
  }
);

export interface BadgeProps
  extends HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof badgeVariants> {}

export function Badge({ className, tone, ...props }: BadgeProps) {
  return (
    <span className={cn(badgeVariants({ tone }), className)} {...props} />
  );
}

export function SurfaceCard({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        "rounded-xl border border-[var(--border)] bg-[var(--card)] text-[var(--card-foreground)]",
        className
      )}
      {...props}
    />
  );
}

export function Input({
  className,
  ...props
}: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      className={cn(
        "flex h-8 w-full rounded-lg border border-[var(--input)] bg-[var(--card)] px-3 py-1 text-xs text-[var(--foreground)] placeholder:text-[var(--muted-foreground)] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[var(--ring)] disabled:cursor-not-allowed disabled:opacity-50",
        className
      )}
      {...props}
    />
  );
}

export function Textarea({
  className,
  ...props
}: TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return (
    <textarea
      className={cn(
        "flex min-h-[68px] w-full rounded-lg border border-[var(--input)] bg-[var(--card)] px-3 py-2 text-xs text-[var(--foreground)] placeholder:text-[var(--muted-foreground)] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[var(--ring)] disabled:cursor-not-allowed disabled:opacity-50",
        className
      )}
      {...props}
    />
  );
}

export interface SegmentedOption<T extends string> {
  value: T;
  label: string;
}

export interface SegmentedToggleProps<T extends string> {
  options: readonly SegmentedOption<T>[];
  value: T;
  onChange: (next: T) => void;
  ariaLabel: string;
  className?: string | undefined;
}

export function SegmentedToggle<T extends string>({
  options,
  value,
  onChange,
  ariaLabel,
  className,
}: SegmentedToggleProps<T>) {
  return (
    <div
      role="group"
      aria-label={ariaLabel}
      className={cn(
        "inline-flex items-center rounded-lg border border-[var(--border)] bg-[var(--secondary)] p-0.5",
        className
      )}
    >
      {options.map((opt) => {
        const active = opt.value === value;
        return (
          <button
            key={opt.value}
            type="button"
            onClick={() => onChange(opt.value)}
            className={cn(
              "rounded-md px-2.5 py-1 text-[11px] font-medium transition-all cursor-pointer",
              active
                ? "bg-[var(--card)] text-[var(--foreground)] shadow-2xs"
                : "text-[var(--muted-foreground)] hover:text-[var(--foreground)]"
            )}
          >
            {opt.label}
          </button>
        );
      })}
    </div>
  );
}

export const Dialog = DialogPrimitive.Root;
export const DialogTrigger = DialogPrimitive.Trigger;
export const DialogClose = DialogPrimitive.Close;

export function DialogContent({
  className,
  children,
  onCloseClick,
  ...props
}: ComponentPropsWithoutRef<typeof DialogPrimitive.Content> & {
  onCloseClick?: (() => void) | undefined;
}) {
  return (
    <DialogPrimitive.Portal>
      <DialogPrimitive.Overlay className="fixed inset-0 z-50 bg-black/40 backdrop-blur-[2px]" />
      <DialogPrimitive.Content
        className={cn(
          "fixed left-1/2 top-1/2 z-50 w-[94vw] max-w-2xl -translate-x-1/2 -translate-y-1/2 rounded-2xl border border-[var(--border)] bg-[var(--popover)] p-0 text-[var(--popover-foreground)] shadow-2xl focus:outline-none",
          className
        )}
        {...props}
      >
        {children}
        {onCloseClick && (
          <button
            type="button"
            onClick={onCloseClick}
            aria-label="Close"
            className="absolute end-4 top-4 rounded-md p-1 text-[var(--muted-foreground)] hover:bg-[var(--secondary)] hover:text-[var(--foreground)] cursor-pointer"
          >
            <X className="h-4 w-4" />
          </button>
        )}
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  );
}

export const DialogTitle = DialogPrimitive.Title;
export const DialogDescription = DialogPrimitive.Description;

export const Tabs = TabsPrimitive.Root;

export function TabsList({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof TabsPrimitive.List>) {
  return (
    <TabsPrimitive.List
      className={cn(
        "inline-flex h-8 items-center justify-center rounded-lg border border-[var(--border)] bg-[var(--secondary)] p-0.5 text-[var(--muted-foreground)]",
        className
      )}
      {...props}
    />
  );
}

export function TabsTrigger({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger>) {
  return (
    <TabsPrimitive.Trigger
      className={cn(
        "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-xs font-medium transition-all cursor-pointer data-[state=active]:bg-[var(--card)] data-[state=active]:text-[var(--foreground)] data-[state=active]:shadow-2xs",
        className
      )}
      {...props}
    />
  );
}

export const TabsContent = TabsPrimitive.Content;

export function Switch({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof SwitchPrimitive.Root>) {
  return (
    <SwitchPrimitive.Root
      className={cn(
        "peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ring)] data-[state=checked]:bg-[var(--emerald)] data-[state=unchecked]:bg-zinc-300 dark:data-[state=unchecked]:bg-zinc-700",
        className
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb className="pointer-events-none block h-4 w-4 rounded-full bg-white shadow-xs transition-transform data-[state=checked]:translate-x-4 rtl:data-[state=checked]:-translate-x-4 data-[state=unchecked]:translate-x-0.5" />
    </SwitchPrimitive.Root>
  );
}

export const Select = SelectPrimitive.Root;
export const SelectValue = SelectPrimitive.Value;

export function SelectTrigger({
  className,
  children,
  ...props
}: ComponentPropsWithoutRef<typeof SelectPrimitive.Trigger>) {
  return (
    <SelectPrimitive.Trigger
      className={cn(
        "inline-flex h-8 items-center justify-between gap-2 rounded-lg border border-[var(--border)] bg-[var(--card)] px-2.5 py-1 text-xs font-medium text-[var(--foreground)] hover:bg-[var(--secondary)] focus:outline-none cursor-pointer",
        className
      )}
      {...props}
    >
      {children}
      <SelectPrimitive.Icon asChild>
        <ChevronDown className="h-3.5 w-3.5 opacity-60" />
      </SelectPrimitive.Icon>
    </SelectPrimitive.Trigger>
  );
}

export function SelectContent({
  className,
  children,
  ...props
}: ComponentPropsWithoutRef<typeof SelectPrimitive.Content>) {
  return (
    <SelectPrimitive.Portal>
      <SelectPrimitive.Content
        position="popper"
        sideOffset={4}
        className={cn(
          "z-50 min-w-[160px] overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--popover)] p-1 text-[var(--popover-foreground)] shadow-lg",
          className
        )}
        {...props}
      >
        <SelectPrimitive.Viewport>{children}</SelectPrimitive.Viewport>
      </SelectPrimitive.Content>
    </SelectPrimitive.Portal>
  );
}

export function SelectItem({
  className,
  children,
  ...props
}: ComponentPropsWithoutRef<typeof SelectPrimitive.Item>) {
  return (
    <SelectPrimitive.Item
      className={cn(
        "relative flex w-full cursor-pointer select-none items-center rounded-lg py-1.5 ps-7 pe-2.5 text-xs outline-none hover:bg-[var(--secondary)] focus:bg-[var(--secondary)]",
        className
      )}
      {...props}
    >
      <span className="absolute start-2 flex h-3.5 w-3.5 items-center justify-center">
        <SelectPrimitive.ItemIndicator>
          <Check className="h-3.5 w-3.5 text-[var(--emerald)]" />
        </SelectPrimitive.ItemIndicator>
      </span>
      <SelectPrimitive.ItemText>{children}</SelectPrimitive.ItemText>
    </SelectPrimitive.Item>
  );
}

export const TooltipProvider = TooltipPrimitive.Provider;
export const Tooltip = TooltipPrimitive.Root;
export const TooltipTrigger = TooltipPrimitive.Trigger;

export function TooltipContent({
  className,
  sideOffset = 6,
  ...props
}: ComponentPropsWithoutRef<typeof TooltipPrimitive.Content>) {
  return (
    <TooltipPrimitive.Portal>
      <TooltipPrimitive.Content
        sideOffset={sideOffset}
        className={cn(
          "z-50 rounded-md bg-[var(--primary)] px-2.5 py-1 text-[11px] font-medium text-[var(--primary-foreground)] shadow-md",
          className
        )}
        {...props}
      />
    </TooltipPrimitive.Portal>
  );
}

export function ScrollArea({
  className,
  children,
  ...props
}: ComponentPropsWithoutRef<typeof ScrollAreaPrimitive.Root>) {
  return (
    <ScrollAreaPrimitive.Root
      className={cn("relative overflow-hidden", className)}
      {...props}
    >
      <ScrollAreaPrimitive.Viewport className="h-full w-full rounded-[inherit]">
        {children}
      </ScrollAreaPrimitive.Viewport>
      <ScrollAreaPrimitive.Scrollbar
        orientation="vertical"
        className="flex touch-none select-none p-0.5 transition-colors w-2"
      >
        <ScrollAreaPrimitive.Thumb className="relative flex-1 rounded-full bg-zinc-300 dark:bg-zinc-700" />
      </ScrollAreaPrimitive.Scrollbar>
    </ScrollAreaPrimitive.Root>
  );
}

export interface CommandPaletteAction {
  id: string;
  label: string;
  shortcut?: string | undefined;
  group: string;
  onSelect: () => void;
}

export interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  placeholder: string;
  emptyText: string;
  actions: readonly CommandPaletteAction[];
}

export function CommandPalette({
  open,
  onOpenChange,
  placeholder,
  emptyText,
  actions,
}: CommandPaletteProps) {
  const groups = Array.from(new Set(actions.map((a) => a.group)));

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg overflow-hidden p-0">
        <DialogTitle className="sr-only">{placeholder}</DialogTitle>
        <CommandPrimitive className="flex h-full w-full flex-col overflow-hidden bg-[var(--popover)] text-[var(--popover-foreground)]">
          <div className="flex items-center gap-2 border-b border-[var(--border)] px-3.5 py-2.5">
            <Search className="h-4 w-4 text-[var(--muted-foreground)] shrink-0" />
            <CommandPrimitive.Input
              placeholder={placeholder}
              className="flex h-6 w-full bg-transparent text-xs outline-none placeholder:text-[var(--muted-foreground)]"
            />
          </div>
          <CommandPrimitive.List className="max-h-72 overflow-y-auto p-1.5">
            <CommandPrimitive.Empty className="py-6 text-center text-xs text-[var(--muted-foreground)]">
              {emptyText}
            </CommandPrimitive.Empty>
            {groups.map((grp) => (
              <CommandPrimitive.Group
                key={grp}
                heading={grp}
                className="px-1 py-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--muted-foreground)]"
              >
                {actions
                  .filter((a) => a.group === grp)
                  .map((act) => (
                    <CommandPrimitive.Item
                      key={act.id}
                      value={`${act.group} ${act.label}`}
                      onSelect={() => {
                        act.onSelect();
                        onOpenChange(false);
                      }}
                      className="flex cursor-pointer items-center justify-between rounded-lg px-2.5 py-2 text-xs font-normal normal-case tracking-normal text-[var(--foreground)] aria-selected:bg-[var(--secondary)]"
                    >
                      <span>{act.label}</span>
                      {act.shortcut && (
                        <kbd className="rounded border border-[var(--border)] bg-[var(--background)] px-1.5 py-0.5 font-mono-tabular text-[10px] text-[var(--muted-foreground)]">
                          {act.shortcut}
                        </kbd>
                      )}
                    </CommandPrimitive.Item>
                  ))}
              </CommandPrimitive.Group>
            ))}
          </CommandPrimitive.List>
        </CommandPrimitive>
      </DialogContent>
    </Dialog>
  );
}

export function QlevelToaster({ dark = false }: { dark?: boolean | undefined }) {
  return (
    <SonnerToaster
      theme={dark ? "dark" : "light"}
      position="bottom-right"
      toastOptions={{
        className: "text-xs font-medium border border-[var(--border)]",
      }}
    />
  );
}

export interface ProgressPillProps {
  percent: number;
  label: string;
  children?: ReactNode | undefined;
}

export function ProgressPill({ percent, label, children }: ProgressPillProps) {
  const clamped = Math.max(0, Math.min(100, percent));
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between text-xs">
        <span className="font-medium text-[var(--foreground)]">{label}</span>
        <span className="font-mono-tabular text-[var(--muted-foreground)]">
          {clamped}%
        </span>
      </div>
      <div className="h-1.5 w-full overflow-hidden rounded-full bg-[var(--secondary)]">
        <div
          className="h-full rounded-full bg-[var(--emerald)] transition-all duration-300"
          style={{ width: `${clamped}%` }}
        />
      </div>
      {children}
    </div>
  );
}
