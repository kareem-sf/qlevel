import {
  Check,
  FileSpreadsheet,
  FileText,
  Loader2,
  Paperclip,
  Sparkles,
  X,
} from "lucide-react";
import type { HTMLAttributes, ReactNode } from "react";
import { cn } from "./lib/utils";

/**
 * Vercel AI Elements (`vercel/ai-elements`) patterns built on top of `shadcn/ui`:
 * - PromptInput / PromptInputBody / PromptInputToolbar / PromptInputPill
 * - Attachments / AttachmentChip
 * - ChainOfThought / ChainOfThoughtStep / Checkpoint
 * - Artifact / ArtifactHeader / Sources / SourceQuoteCard
 */

export function PromptInput({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        "rounded-2xl border border-[var(--border)] bg-[var(--card)] shadow-sm transition-shadow focus-within:shadow-md",
        className
      )}
      {...props}
    />
  );
}

export function PromptInputBody({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("p-4 space-y-3", className)} {...props} />;
}

export function PromptInputToolbar({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        "flex flex-wrap items-center justify-between gap-2 border-t border-[var(--border)] bg-[var(--background)]/50 px-4 py-2.5 rounded-b-2xl",
        className
      )}
      {...props}
    />
  );
}

export interface PromptInputPillProps {
  label: string;
  onClick?: (() => void) | undefined;
  active?: boolean | undefined;
  icon?: ReactNode | undefined;
}

export function PromptInputPill({
  label,
  onClick,
  active = false,
  icon,
}: PromptInputPillProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[11px] font-medium transition-colors cursor-pointer",
        active
          ? "border-[var(--foreground)] bg-[var(--foreground)] text-[var(--background)]"
          : "border-[var(--border)] bg-[var(--secondary)]/80 text-[var(--secondary-foreground)] hover:bg-[var(--secondary)]"
      )}
    >
      {icon}
      <span dir="auto">{label}</span>
    </button>
  );
}

export function Attachments({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("flex flex-wrap items-center gap-2", className)}
      {...props}
    />
  );
}

export interface AttachmentChipProps {
  fileName: string;
  roleLabel: string;
  isSpreadsheet?: boolean | undefined;
  warningLabel?: string | undefined;
  vendorName?: string | undefined;
  showVendorInput?: boolean | undefined;
  vendorPlaceholder?: string | undefined;
  onChangeVendorName?: ((val: string) => void) | undefined;
  onCycleRole?: (() => void) | undefined;
  onRemove?: (() => void) | undefined;
}

export function AttachmentChip({
  fileName,
  roleLabel,
  isSpreadsheet = false,
  warningLabel,
  vendorName,
  showVendorInput = false,
  vendorPlaceholder,
  onChangeVendorName,
  onCycleRole,
  onRemove,
}: AttachmentChipProps) {
  const Icon = isSpreadsheet ? FileSpreadsheet : FileText;
  return (
    <div className="group inline-flex flex-wrap items-center gap-1.5 rounded-lg border border-[var(--border)] bg-[var(--secondary)]/65 px-2.5 py-1.5 text-xs transition-colors hover:border-zinc-400 dark:hover:border-zinc-600">
      <Icon className="h-3.5 w-3.5 text-[var(--muted-foreground)] shrink-0" />
      <span dir="auto" className="font-medium text-[var(--foreground)]">
        {fileName}
      </span>
      <span className="text-[var(--muted-foreground)]">·</span>
      <button
        type="button"
        onClick={onCycleRole}
        className="rounded px-1.5 py-0.5 text-[11px] font-medium text-[var(--muted-foreground)] hover:bg-[var(--card)] hover:text-[var(--foreground)] cursor-pointer transition-colors"
      >
        {roleLabel}
      </button>
      {showVendorInput && onChangeVendorName && (
        <input
          dir="auto"
          type="text"
          value={vendorName ?? ""}
          onChange={(e) => onChangeVendorName(e.target.value)}
          placeholder={vendorPlaceholder}
          className="h-5 w-32 rounded border border-transparent bg-[var(--card)]/80 px-1.5 text-[11px] text-[var(--foreground)] placeholder:text-[var(--muted-foreground)] focus:border-[var(--border)] focus:outline-none"
        />
      )}
      {warningLabel && (
        <span className="inline-flex items-center gap-1 text-[11px] font-medium text-[var(--amber-fg)]">
          <span className="h-1.5 w-1.5 rounded-full bg-[var(--amber)]" />
          {warningLabel}
        </span>
      )}
      {onRemove && (
        <button
          type="button"
          onClick={onRemove}
          aria-label="Remove file"
          className="ms-0.5 rounded p-0.5 text-[var(--muted-foreground)] opacity-60 hover:bg-[var(--card)] hover:text-[var(--foreground)] hover:opacity-100 cursor-pointer"
        >
          <X className="h-3 w-3" />
        </button>
      )}
    </div>
  );
}

export function ChainOfThought({
  title,
  live = false,
  children,
  className,
}: {
  title: string;
  live?: boolean | undefined;
  children: ReactNode;
  className?: string | undefined;
}) {
  return (
    <div className={cn("space-y-3", className)}>
      <div className="flex items-center justify-between">
        <div className="inline-flex items-center gap-2 text-xs font-semibold text-[var(--foreground)]">
          <Sparkles className="h-3.5 w-3.5 text-[var(--emerald)]" />
          <span>{title}</span>
        </div>
        {live && (
          <span className="inline-flex items-center gap-1.5 rounded-full bg-[var(--emerald-subtle)] px-2 py-0.5 text-[10px] font-medium text-[var(--emerald-fg)]">
            <span className="h-1.5 w-1.5 rounded-full bg-[var(--emerald)] animate-pulse" />
            Live
          </span>
        )}
      </div>
      <div className="relative space-y-2.5 before:absolute before:start-[7px] before:top-2 before:bottom-2 before:w-px before:bg-[var(--border)]">
        {children}
      </div>
    </div>
  );
}

export interface ChainOfThoughtStepProps {
  label: string;
  detail?: string | undefined;
  status: "done" | "active" | "pending";
}

export function ChainOfThoughtStep({
  label,
  detail,
  status,
}: ChainOfThoughtStepProps) {
  return (
    <div className="relative flex items-start gap-2.5 ps-5 text-xs">
      <span
        className={cn(
          "absolute start-0 top-0.5 flex h-3.5 w-3.5 items-center justify-center rounded-full border bg-[var(--card)]",
          status === "done"
            ? "border-[var(--emerald)] text-[var(--emerald)]"
            : status === "active"
              ? "border-[var(--foreground)] text-[var(--foreground)]"
              : "border-[var(--border)] text-[var(--muted-foreground)]"
        )}
      >
        {status === "done" ? (
          <Check className="h-2.5 w-2.5" />
        ) : status === "active" ? (
          <Loader2 className="h-2.5 w-2.5 animate-spin" />
        ) : (
          <span className="h-1 w-1 rounded-full bg-[var(--muted-foreground)]" />
        )}
      </span>
      <div className="space-y-0.5">
        <p
          className={cn(
            "leading-snug",
            status === "pending"
              ? "text-[var(--muted-foreground)]"
              : "font-medium text-[var(--foreground)]"
          )}
        >
          {label}
        </p>
        {detail && (
          <p dir="auto" className="text-[11px] text-[var(--muted-foreground)]">
            {detail}
          </p>
        )}
      </div>
    </div>
  );
}

export function Checkpoint({ label }: { label: string }) {
  return (
    <div className="inline-flex items-center gap-1.5 rounded-full border border-[var(--border)] bg-[var(--card)] px-2.5 py-1 text-[11px] text-[var(--muted-foreground)]">
      <Paperclip className="h-3 w-3 text-[var(--emerald)]" />
      <span>{label}</span>
    </div>
  );
}

export function Artifact({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        "flex flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--card)]",
        className
      )}
      {...props}
    />
  );
}

export function ArtifactHeader({
  title,
  subtitle,
  actions,
}: {
  title: string;
  subtitle?: string | undefined;
  actions?: ReactNode | undefined;
}) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--border)] px-4 py-3">
      <div className="space-y-0.5">
        <h3 className="text-xs font-semibold text-[var(--foreground)]">
          {title}
        </h3>
        {subtitle && (
          <p dir="auto" className="text-[11px] text-[var(--muted-foreground)]">
            {subtitle}
          </p>
        )}
      </div>
      {actions && <div className="flex items-center gap-2">{actions}</div>}
    </div>
  );
}

export interface SourceQuoteCardProps {
  vendorName: string;
  title: string;
  category: string;
  quoteSnippet?: string | null | undefined;
  impactLabel?: string | null | undefined;
  detail: string;
}

export function SourceQuoteCard({
  vendorName,
  title,
  category,
  quoteSnippet,
  impactLabel,
  detail,
}: SourceQuoteCardProps) {
  return (
    <div className="space-y-2.5 rounded-xl border border-[var(--border)] bg-[var(--background)] p-3.5 text-xs">
      <div className="flex flex-wrap items-center justify-between gap-1.5">
        <span dir="auto" className="font-semibold text-[var(--foreground)]">
          {vendorName}
        </span>
        <span className="rounded-full border border-[var(--border)] bg-[var(--card)] px-2 py-0.5 text-[10px] font-medium text-[var(--muted-foreground)]">
          {category}
        </span>
      </div>
      <p className="font-medium text-[var(--foreground)]">{title}</p>
      {quoteSnippet && (
        <blockquote
          dir="auto"
          className="rounded-lg border border-amber-500/25 bg-[var(--amber-subtle)] px-3 py-2 text-[11px] leading-relaxed text-[var(--amber-fg)]"
        >
          "{quoteSnippet}"
        </blockquote>
      )}
      <p className="text-[11px] leading-relaxed text-[var(--muted-foreground)]">
        {detail}
      </p>
      {impactLabel && (
        <div className="pt-0.5">
          <span
            dir="ltr"
            className="inline-flex items-center rounded-full border border-[var(--border)] bg-[var(--card)] px-2.5 py-0.5 font-mono-tabular text-[11px] font-semibold text-[var(--foreground)]"
          >
            {impactLabel}
          </span>
        </div>
      )}
    </div>
  );
}
