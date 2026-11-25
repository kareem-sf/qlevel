import type {
  DocumentRoleDto,
  LevelingSessionConfig,
  StagedInputDocumentDto,
} from "@qlevel/bindings";
import {
  AttachmentChip,
  Attachments,
  Button,
  PromptInput,
  PromptInputBody,
  PromptInputPill,
  PromptInputToolbar,
  QlevelBrandLockup,
  cn,
} from "@qlevel/design-system";
import { Paperclip, Sparkles } from "lucide-react";
import type { Dictionary, UiLanguage } from "../i18n";

export interface AgenticIntakeViewProps {
  dict: Dictionary;
  uiLang: UiLanguage;
  config: LevelingSessionConfig;
  documents: readonly StagedInputDocumentDto[];
  isDragging: boolean;
  anyProviderConnected: boolean;
  busy: boolean;
  onBrowseFiles: () => void;
  onChangeRole: (id: string, next: DocumentRoleDto) => void;
  onChangeVendorName: (id: string, name: string) => void;
  onRemoveDocument: (id: string) => void;
  onToggleLanguage: () => void;
  onToggleLayout: () => void;
  onOpenSettings: (initialTab?: "providers" | "rules" | "excel") => void;
  onStartLeveling: () => void;
}

const ROLE_ORDER: readonly DocumentRoleDto[] = [
  "master_boq",
  "vendor_quote",
  "spec_or_avl",
];

export function AgenticIntakeView({
  dict,
  uiLang,
  config,
  documents,
  isDragging,
  anyProviderConnected,
  busy,
  onBrowseFiles,
  onChangeRole,
  onChangeVendorName,
  onRemoveDocument,
  onToggleLanguage,
  onToggleLayout,
  onOpenSettings,
  onStartLeveling,
}: AgenticIntakeViewProps) {
  const hasMasterBoq = documents.some((d) => d.role === "master_boq");
  const hasVendorQuote = documents.some((d) => d.role === "vendor_quote");

  const roleText = (r: DocumentRoleDto): string => {
    if (r === "master_boq") return dict.roleMaster;
    if (r === "vendor_quote") return dict.roleVendor;
    return dict.roleAvl;
  };

  const modelPillText =
    config.ai.modelId === null || config.ai.modelId === "auto"
      ? dict.autoMultimodalPill
      : config.ai.modelId;

  return (
    <div className="flex flex-1 flex-col bg-[var(--background)]">
      {/* Minimal Top Bar */}
      <header className="flex items-center justify-between px-6 py-4">
        <QlevelBrandLockup version="v0.0.2" />
        <button
          type="button"
          onClick={() => onOpenSettings("providers")}
          className="inline-flex items-center gap-2 rounded-full border border-[var(--border)] bg-[var(--card)] px-3 py-1 text-[11px] font-medium text-[var(--muted-foreground)] hover:text-[var(--foreground)] transition-colors cursor-pointer"
        >
          <span
            className={cn(
              "h-1.5 w-1.5 rounded-full",
              anyProviderConnected ? "bg-[var(--emerald)]" : "bg-[var(--amber)]"
            )}
          />
          <span>
            {anyProviderConnected
              ? dict.connectedLabel
              : dict.disconnectedLabel}
          </span>
        </button>
      </header>

      {/* Centered Minimalist Composer Canvas */}
      <div className="flex flex-1 items-center justify-center px-6 pb-12">
        <div className="w-full max-w-2xl space-y-3">
          <PromptInput
            className={cn(
              "p-1 transition-colors",
              isDragging && "border-[var(--emerald)] ring-2 ring-emerald-500/20"
            )}
          >
            <PromptInputBody className="space-y-4 p-5">
              <div className="flex items-center justify-between gap-3">
                <h1 className="text-lg font-semibold tracking-tight text-[var(--foreground)]">
                  {dict.intakeHeadline}
                </h1>
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={busy}
                  onClick={onBrowseFiles}
                >
                  <Paperclip className="h-3.5 w-3.5" />
                  <span>{dict.browseBtn}</span>
                </Button>
              </div>

              {/* Clean Drop Target Box */}
              <div
                onClick={onBrowseFiles}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    onBrowseFiles();
                  }
                }}
                className="flex min-h-[104px] cursor-pointer flex-col justify-between rounded-xl border border-[var(--border)] bg-[var(--background)]/45 p-3.5 transition-colors hover:border-zinc-400 dark:hover:border-zinc-600"
              >
                <p className="text-xs text-[var(--muted-foreground)]">
                  {dict.intakePlaceholder}
                </p>
                <p className="text-[11px] text-[var(--muted-foreground)]/75">
                  {dict.pasteShortcutHint}
                </p>
              </div>

              {/* Staged File Pills */}
              {documents.length > 0 && (
                <Attachments>
                  {documents.map((doc) => {
                    const nextRole = () => {
                      const idx = ROLE_ORDER.indexOf(doc.role);
                      const next = ROLE_ORDER[(idx + 1) % ROLE_ORDER.length];
                      if (next) {
                        onChangeRole(doc.id, next);
                      }
                    };
                    const isExcel = /\.(xlsx|xls|xlsm|xlsb|csv|tsv)$/i.test(
                      doc.fileName
                    );
                    return (
                      <AttachmentChip
                        key={doc.id}
                        fileName={doc.fileName}
                        roleLabel={roleText(doc.role)}
                        isSpreadsheet={isExcel}
                        warningLabel={
                          doc.hiddenRowsCount > 0
                            ? `${doc.hiddenRowsCount} ${dict.hiddenRowsLabel}`
                            : undefined
                        }
                        showVendorInput={doc.role === "vendor_quote"}
                        vendorName={doc.vendorName ?? ""}
                        vendorPlaceholder={dict.vendorPlaceholder}
                        onChangeVendorName={(v) =>
                          onChangeVendorName(doc.id, v)
                        }
                        onCycleRole={nextRole}
                        onRemove={() => onRemoveDocument(doc.id)}
                      />
                    );
                  })}
                </Attachments>
              )}

              {!hasMasterBoq && hasVendorQuote && (
                <p className="text-[11px] font-medium text-[var(--emerald-fg)]">
                  {dict.syntheticBaselineNotice}
                </p>
              )}
            </PromptInputBody>

            {/* Bottom Quick-Config Pills & Obsidian Primary Action */}
            <PromptInputToolbar>
              <div className="flex flex-wrap items-center gap-1.5">
                <PromptInputPill
                  label={modelPillText}
                  onClick={() => onOpenSettings("providers")}
                />
                <PromptInputPill
                  label={`${config.currency} · ${config.vatPolicy.projectVatPct}% VAT`}
                  onClick={() => onOpenSettings("rules")}
                />
                <PromptInputPill
                  label={
                    config.layoutMode === "single_master_sheet"
                      ? dict.singleSheetPill
                      : dict.dualTabPill
                  }
                  onClick={onToggleLayout}
                />
                <PromptInputPill
                  label={uiLang === "en" ? "EN / AR" : "AR / EN"}
                  onClick={onToggleLanguage}
                />
              </div>

              <Button
                variant="primary"
                size="lg"
                disabled={busy || !hasVendorQuote}
                onClick={onStartLeveling}
              >
                <span>{dict.startLevelingBtn}</span>
                <Sparkles className="h-3.5 w-3.5 text-[var(--emerald)]" />
              </Button>
            </PromptInputToolbar>
          </PromptInput>
        </div>
      </div>
    </div>
  );
}
