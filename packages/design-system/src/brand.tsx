import { cn } from "./lib/utils";

export interface QlevelMarkProps {
  size?: number | undefined;
  className?: string | undefined;
}

export function QlevelMark({ size = 28, className }: QlevelMarkProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 512 512"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
      className={cn("shrink-0", className)}
    >
      <rect x="16" y="16" width="480" height="480" rx="112" fill="#09090B" />
      <rect
        x="20"
        y="20"
        width="472"
        height="472"
        rx="108"
        stroke="#27272A"
        strokeWidth="8"
      />
      <rect x="128" y="196" width="60" height="172" rx="8" fill="#FAFAFA" />
      <rect x="226" y="124" width="60" height="206" rx="8" fill="#FAFAFA" />
      <rect x="324" y="244" width="60" height="124" rx="8" fill="#FAFAFA" />
      <path
        d="M96 368H220C224 347 238 334 256 334C274 334 288 347 292 368H416"
        stroke="#FAFAFA"
        strokeWidth="16"
        strokeLinecap="round"
      />
      <circle cx="256" cy="368" r="22" fill="#10B981" />
    </svg>
  );
}

export interface QlevelBrandLockupProps {
  subtitle?: string | undefined;
  version?: string | undefined;
  className?: string | undefined;
}

export function QlevelBrandLockup({
  subtitle,
  version,
  className,
}: QlevelBrandLockupProps) {
  return (
    <div className={cn("inline-flex items-center gap-2.5", className)}>
      <QlevelMark size={26} />
      <div className="flex items-baseline gap-2">
        <span className="text-[15px] font-semibold tracking-tight text-[var(--foreground)]">
          qlevel
        </span>
        {version && (
          <span className="rounded-full border border-[var(--border)] bg-[var(--secondary)] px-1.5 py-0.2 font-mono-tabular text-[10px] font-medium text-[var(--muted-foreground)]">
            {version}
          </span>
        )}
        {subtitle && (
          <>
            <span className="text-xs text-[var(--border)]">|</span>
            <span className="text-xs font-medium text-[var(--muted-foreground)]">
              {subtitle}
            </span>
          </>
        )}
      </div>
    </div>
  );
}
