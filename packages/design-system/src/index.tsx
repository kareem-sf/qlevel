import type { ButtonHTMLAttributes, ReactNode } from "react";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant | undefined;
  children: ReactNode;
}

const BUTTON_STYLES: Record<ButtonVariant, string> = {
  primary:
    "bg-teal-700 text-white hover:bg-teal-800 focus-visible:outline-teal-700 shadow-xs",
  secondary:
    "bg-white text-slate-800 border border-slate-300 hover:bg-slate-50 focus-visible:outline-slate-600",
  ghost:
    "bg-transparent text-slate-600 hover:bg-slate-100 hover:text-slate-900 focus-visible:outline-slate-500",
  danger:
    "bg-red-700 text-white hover:bg-red-800 focus-visible:outline-red-700",
};

export function Button({
  variant = "primary",
  className = "",
  disabled,
  children,
  ...rest
}: ButtonProps) {
  return (
    <button
      type="button"
      disabled={disabled}
      className={`inline-flex items-center justify-center gap-2 rounded-lg px-4 py-2 text-sm font-semibold transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:opacity-50 ${BUTTON_STYLES[variant]} ${className}`}
      {...rest}
    >
      {children}
    </button>
  );
}

export type BadgeTone = "neutral" | "success" | "warning" | "danger" | "accent";

export interface BadgeProps {
  tone?: BadgeTone | undefined;
  children: ReactNode;
}

const BADGE_STYLES: Record<BadgeTone, string> = {
  neutral: "bg-slate-100 text-slate-700 border-slate-200",
  success: "bg-emerald-50 text-emerald-800 border-emerald-200",
  warning: "bg-amber-50 text-amber-800 border-amber-200",
  danger: "bg-red-50 text-red-800 border-red-200",
  accent: "bg-teal-50 text-teal-800 border-teal-200",
};

export function Badge({ tone = "neutral", children }: BadgeProps) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-md border px-2.5 py-0.5 text-xs font-medium ${BADGE_STYLES[tone]}`}
    >
      {children}
    </span>
  );
}

export interface SurfaceCardProps {
  children: ReactNode;
  className?: string | undefined;
}

export function SurfaceCard({ children, className = "" }: SurfaceCardProps) {
  return (
    <section
      className={`rounded-xl border border-slate-200 bg-white p-5 shadow-xs ${className}`}
    >
      {children}
    </section>
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
}

export function SegmentedToggle<T extends string>({
  options,
  value,
  onChange,
  ariaLabel,
}: SegmentedToggleProps<T>) {
  return (
    <div
      role="group"
      aria-label={ariaLabel}
      className="inline-flex rounded-lg border border-slate-200 bg-slate-100 p-0.5"
    >
      {options.map((opt) => {
        const active = opt.value === value;
        return (
          <button
            key={opt.value}
            type="button"
            onClick={() => onChange(opt.value)}
            className={`rounded-md px-3 py-1.5 text-xs font-semibold transition-colors ${
              active
                ? "bg-white text-slate-900 shadow-xs"
                : "text-slate-600 hover:text-slate-900"
            }`}
          >
            {opt.label}
          </button>
        );
      })}
    </div>
  );
}

export interface CollapsibleSectionProps {
  title: string;
  summaryText: string;
  open: boolean;
  onToggle: () => void;
  children: ReactNode;
}

export function CollapsibleSection({
  title,
  summaryText,
  open,
  onToggle,
  children,
}: CollapsibleSectionProps) {
  return (
    <div className="rounded-xl border border-slate-200 bg-white">
      <button
        type="button"
        aria-expanded={open}
        onClick={onToggle}
        className="flex w-full items-center justify-between gap-4 px-5 py-3.5 text-start hover:bg-slate-50"
      >
        <div className="flex flex-wrap items-center gap-2.5">
          <span className="text-sm font-semibold text-slate-800">{title}</span>
          {!open && (
            <span className="rounded-md bg-slate-100 px-2.5 py-0.5 text-xs text-slate-600">
              {summaryText}
            </span>
          )}
        </div>
        <span className="text-xs font-medium text-teal-700">
          {open ? "-" : "+"}
        </span>
      </button>
      {open && (
        <div className="border-t border-slate-200 px-5 py-4">{children}</div>
      )}
    </div>
  );
}

export interface ProgressBarProps {
  percent: number;
  label: string;
  sublabel?: string | undefined;
}

export function ProgressBar({ percent, label, sublabel }: ProgressBarProps) {
  const clamped = Math.max(0, Math.min(100, percent));
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between text-xs font-medium text-slate-700">
        <span>{label}</span>
        <span className="tabular-nums">{clamped}%</span>
      </div>
      <div className="h-2.5 w-full overflow-hidden rounded-full bg-slate-200">
        <div
          className="h-full rounded-full bg-teal-700 transition-all duration-300"
          style={{ width: `${clamped}%` }}
        />
      </div>
      {sublabel && <p className="text-xs text-slate-500">{sublabel}</p>}
    </div>
  );
}
