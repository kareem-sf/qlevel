import type {
  CurrencyCode,
  EffortLevel,
  LevelingSessionConfig,
  MergedItemMode,
  ModelInfo,
  PeerPlugStrategy,
  ProviderAuthStatusDto,
  ProviderKind,
  SheetDirection,
  WorkbookLayoutMode,
} from "@qlevel/bindings";
import {
  Badge,
  Button,
  Dialog,
  DialogContent,
  DialogTitle,
  Input,
  SegmentedToggle,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Textarea,
  cn,
} from "@qlevel/design-system";
import { Eye, EyeOff, RefreshCw } from "lucide-react";
import { useState } from "react";
import type { Dictionary } from "../i18n";

export type SettingsTabId = "providers" | "rules" | "excel";

export interface SettingsCommandPopoverProps {
  open: boolean;
  initialTab: SettingsTabId;
  dict: Dictionary;
  config: LevelingSessionConfig;
  statuses: readonly ProviderAuthStatusDto[];
  liveModels: readonly ModelInfo[];
  loadingModels: boolean;
  avlText: string;
  customRulesText: string;
  busy: boolean;
  onOpenChange: (open: boolean) => void;
  onChangeTab: (tab: SettingsTabId) => void;
  onChangeConfig: (next: LevelingSessionConfig) => void;
  onChangeAvlText: (val: string) => void;
  onChangeCustomRulesText: (val: string) => void;
  onRefreshModels: () => void;
  onPickRulesFile: () => void;
  onSignInChatGpt: () => void;
  onSaveApiKey: (
    provider: ProviderKind,
    apiKey: string,
    ackGeminiPrivacy: boolean
  ) => void;
  onDisconnect: (provider: ProviderKind) => void;
}

const CURRENCIES: readonly { code: CurrencyCode; vat: string }[] = [
  { code: "SAR", vat: "15" },
  { code: "EGP", vat: "14" },
  { code: "AED", vat: "5" },
  { code: "QAR", vat: "0" },
  { code: "KWD", vat: "0" },
  { code: "BHD", vat: "5" },
  { code: "OMR", vat: "5" },
  { code: "USD", vat: "0" },
  { code: "EUR", vat: "0" },
];

export function SettingsCommandPopover({
  open,
  initialTab,
  dict,
  config,
  statuses,
  liveModels,
  loadingModels,
  avlText,
  customRulesText,
  busy,
  onOpenChange,
  onChangeTab,
  onChangeConfig,
  onChangeAvlText,
  onChangeCustomRulesText,
  onRefreshModels,
  onPickRulesFile,
  onSignInChatGpt,
  onSaveApiKey,
  onDisconnect,
}: SettingsCommandPopoverProps) {
  const [draftKeys, setDraftKeys] = useState<Record<ProviderKind, string>>({
    open_ai: "",
    gemini: "",
    claude: "",
  });
  const [revealKey, setRevealKey] = useState<Record<ProviderKind, boolean>>({
    open_ai: false,
    gemini: false,
    claude: false,
  });
  const [geminiAck, setGeminiAck] = useState<boolean>(false);

  const getStatus = (p: ProviderKind) =>
    statuses.find((s) => s.provider === p);

  const navItems: readonly { id: SettingsTabId; label: string }[] = [
    { id: "providers", label: dict.settingsTabProviders },
    { id: "rules", label: dict.settingsTabRules },
    { id: "excel", label: dict.settingsTabExcel },
  ];

  const renderKeyRow = (provider: ProviderKind, label: string, glyph: string) => {
    const st = getStatus(provider);
    const connected = st?.connected ?? false;
    const keyVal = draftKeys[provider];
    const shown = revealKey[provider];
    const effectiveGeminiAck =
      geminiAck || (st?.geminiPrivacyAcknowledged ?? false);

    return (
      <div
        key={provider}
        className="flex flex-wrap items-center justify-between gap-3 py-3"
      >
        <div className="flex items-center gap-2.5 min-w-[155px]">
          <span className="flex h-6 w-6 items-center justify-center rounded-md border border-[var(--border)] bg-[var(--secondary)] font-mono-tabular text-[11px] font-bold">
            {glyph}
          </span>
          <span className="text-xs font-medium text-[var(--foreground)]">
            {label}
          </span>
          {connected && <Badge tone="success">{dict.connectedLabel}</Badge>}
        </div>

        <div className="flex flex-1 flex-wrap items-center justify-end gap-2">
          {provider === "open_ai" && (
            <Button
              variant="secondary"
              size="sm"
              disabled={busy}
              onClick={onSignInChatGpt}
            >
              {dict.signInChatGpt}
            </Button>
          )}

          <div className="relative flex min-w-[170px] flex-1 max-w-[230px] items-center">
            <Input
              dir="ltr"
              type={shown ? "text" : "password"}
              value={keyVal}
              onChange={(e) =>
                setDraftKeys((prev) => ({
                  ...prev,
                  [provider]: e.target.value,
                }))
              }
              placeholder={
                connected ? "••••••••••••••••" : dict.apiKeyPlaceholder
              }
              className="pe-7 h-7 text-[11px]"
            />
            <button
              type="button"
              onClick={() =>
                setRevealKey((prev) => ({
                  ...prev,
                  [provider]: !prev[provider],
                }))
              }
              className="absolute end-2 text-[var(--muted-foreground)] hover:text-[var(--foreground)] cursor-pointer"
            >
              {shown ? (
                <EyeOff className="h-3.5 w-3.5" />
              ) : (
                <Eye className="h-3.5 w-3.5" />
              )}
            </button>
          </div>

          {provider === "gemini" && (
            <div
              title={dict.geminiPrivacyHint}
              className="inline-flex items-center gap-1.5 text-[11px] text-[var(--muted-foreground)]"
            >
              <span>{dict.geminiPrivacyShort}</span>
              <Switch
                checked={effectiveGeminiAck}
                onCheckedChange={setGeminiAck}
              />
            </div>
          )}

          {keyVal.trim().length > 0 && (
            <Button
              variant="primary"
              size="sm"
              disabled={busy}
              onClick={() => {
                onSaveApiKey(
                  provider,
                  keyVal,
                  provider === "gemini" ? effectiveGeminiAck : true
                );
                setDraftKeys((prev) => ({ ...prev, [provider]: "" }));
              }}
            >
              {dict.saveKeyBtn}
            </Button>
          )}

          {connected && (
            <Button
              variant="ghost"
              size="sm"
              disabled={busy}
              onClick={() => onDisconnect(provider)}
            >
              {dict.disconnectBtn}
            </Button>
          )}
        </div>
      </div>
    );
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent onCloseClick={() => onOpenChange(false)}>
        <div className="border-b border-[var(--border)] px-6 py-4">
          <DialogTitle className="text-base font-semibold tracking-tight text-[var(--foreground)]">
            {dict.settingsTitle}
          </DialogTitle>
        </div>

        <div className="grid min-h-[270px] grid-cols-1 sm:grid-cols-[150px_1fr]">
          {/* Left Tab Rail */}
          <nav className="flex sm:flex-col gap-1 border-b sm:border-b-0 sm:border-e border-[var(--border)] bg-[var(--background)]/50 p-3">
            {navItems.map((item) => {
              const active = initialTab === item.id;
              return (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => onChangeTab(item.id)}
                  className={cn(
                    "rounded-lg px-3 py-1.5 text-start text-xs font-medium transition-colors cursor-pointer",
                    active
                      ? "bg-[var(--secondary)] text-[var(--foreground)]"
                      : "text-[var(--muted-foreground)] hover:text-[var(--foreground)]"
                  )}
                >
                  {item.label}
                </button>
              );
            })}
          </nav>

          {/* Right Tab Content */}
          <div className="p-5">
            {initialTab === "providers" && (
              <div className="space-y-3">
                <div className="divide-y divide-[var(--border)]">
                  {renderKeyRow("open_ai", "OpenAI", "OA")}
                  {renderKeyRow("gemini", "Google Gemini", "G")}
                  {renderKeyRow("claude", "Anthropic Claude", "A\\")}
                </div>

                <div className="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-[var(--border)]">
                  <Select
                    value={config.ai.modelId ?? "auto"}
                    onValueChange={(val) =>
                      onChangeConfig({
                        ...config,
                        ai: {
                          ...config.ai,
                          modelId: val === "auto" ? null : val,
                        },
                      })
                    }
                  >
                    <SelectTrigger className="min-w-[220px]">
                      <SelectValue placeholder={dict.autoMultimodalPill} />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="auto">
                        {dict.autoMultimodalPill}
                      </SelectItem>
                      {liveModels.map((m) => (
                        <SelectItem key={`${m.provider}-${m.id}`} value={m.id}>
                          {m.displayName} ({m.id})
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>

                  <Button
                    variant="ghost"
                    size="sm"
                    disabled={loadingModels}
                    onClick={onRefreshModels}
                  >
                    <RefreshCw
                      className={cn(
                        "h-3.5 w-3.5",
                        loadingModels && "animate-spin"
                      )}
                    />
                    <span>Models</span>
                  </Button>
                </div>
              </div>
            )}

            {initialTab === "rules" && (
              <div className="space-y-4">
                <div className="space-y-1.5">
                  <label className="block text-xs font-medium text-[var(--foreground)]">
                    {dict.avlLabel}
                  </label>
                  <Input
                    dir="auto"
                    value={avlText}
                    onChange={(e) => onChangeAvlText(e.target.value)}
                    placeholder={dict.avlPlaceholder}
                  />
                </div>

                <div className="space-y-1.5">
                  <div className="flex items-center justify-between">
                    <label className="block text-xs font-medium text-[var(--foreground)]">
                      {dict.customRulesLabel}
                    </label>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={onPickRulesFile}
                    >
                      {dict.loadRulesBtn}
                    </Button>
                  </div>
                  <Textarea
                    dir="auto"
                    rows={3}
                    value={customRulesText}
                    onChange={(e) => onChangeCustomRulesText(e.target.value)}
                    placeholder={dict.customRulesPlaceholder}
                  />
                </div>

                <div className="space-y-1.5">
                  <label className="block text-xs font-medium text-[var(--foreground)]">
                    {dict.mergedBehaviorLabel}
                  </label>
                  <SegmentedToggle<MergedItemMode>
                    ariaLabel={dict.mergedBehaviorLabel}
                    value={config.mergedItemMode}
                    onChange={(mergedItemMode) =>
                      onChangeConfig({ ...config, mergedItemMode })
                    }
                    options={[
                      {
                        value: "interactive_review",
                        label: dict.mergedAskInline,
                      },
                      {
                        value: "yellow_highlight_auto",
                        label: dict.mergedAutoHighlight,
                      },
                    ]}
                  />
                </div>
              </div>
            )}

            {initialTab === "excel" && (
              <div className="space-y-4">
                <div className="space-y-1.5">
                  <label className="block text-xs font-medium text-[var(--foreground)]">
                    Layout
                  </label>
                  <SegmentedToggle<WorkbookLayoutMode>
                    ariaLabel="Workbook Layout"
                    value={config.layoutMode}
                    onChange={(layoutMode) =>
                      onChangeConfig({ ...config, layoutMode })
                    }
                    options={[
                      {
                        value: "single_master_sheet",
                        label: dict.singleSheetPill,
                      },
                      { value: "dual_tab", label: dict.dualTabPill },
                    ]}
                  />
                </div>

                <div className="space-y-1.5">
                  <label className="block text-xs font-medium text-[var(--foreground)]">
                    {dict.sheetDirectionLabel}
                  </label>
                  <SegmentedToggle<SheetDirection>
                    ariaLabel={dict.sheetDirectionLabel}
                    value={config.sheetDirection}
                    onChange={(sheetDirection) =>
                      onChangeConfig({ ...config, sheetDirection })
                    }
                    options={[
                      { value: "auto", label: "Auto" },
                      { value: "ltr", label: "LTR" },
                      { value: "rtl", label: "RTL" },
                    ]}
                  />
                </div>

                <div className="grid grid-cols-2 gap-3">
                  <div className="space-y-1.5">
                    <label className="block text-xs font-medium text-[var(--foreground)]">
                      {dict.vatPctLabel}
                    </label>
                    <Input
                      dir="ltr"
                      inputMode="decimal"
                      value={config.vatPolicy.projectVatPct}
                      onChange={(e) =>
                        onChangeConfig({
                          ...config,
                          vatPolicy: {
                            ...config.vatPolicy,
                            projectVatPct: e.target.value,
                          },
                        })
                      }
                      className="font-mono-tabular"
                    />
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Sleek Bottom Quick-Toggle Footer Bar */}
        <div className="flex flex-wrap items-center justify-between gap-3 border-t border-[var(--border)] bg-[var(--background)]/65 px-5 py-3 rounded-b-2xl text-[11px]">
          <div className="flex items-center gap-2">
            <span className="font-medium text-[var(--muted-foreground)]">
              {dict.effortLabel}
            </span>
            <SegmentedToggle<EffortLevel>
              ariaLabel={dict.effortLabel}
              value={config.ai.effort}
              onChange={(effort) =>
                onChangeConfig({ ...config, ai: { ...config.ai, effort } })
              }
              options={[
                { value: "auto", label: dict.effortAuto },
                { value: "low", label: dict.effortLow },
                { value: "medium", label: dict.effortMed },
                { value: "high", label: dict.effortHigh },
              ]}
            />
          </div>

          <div className="flex items-center gap-2">
            <span className="font-medium text-[var(--muted-foreground)]">
              {dict.plugBasisLabel}
            </span>
            <SegmentedToggle<PeerPlugStrategy>
              ariaLabel={dict.plugBasisLabel}
              value={config.peerPlugStrategy}
              onChange={(peerPlugStrategy) =>
                onChangeConfig({ ...config, peerPlugStrategy })
              }
              options={[
                { value: "lowest_compliant", label: dict.plugLowest },
                { value: "average_compliant", label: dict.plugAvg },
                { value: "highest_compliant", label: dict.plugHigh },
                { value: "pte_estimate", label: dict.plugPte },
              ]}
            />
          </div>

          <div className="flex items-center gap-2">
            <span className="font-medium text-[var(--muted-foreground)]">
              {dict.currencyLabel}
            </span>
            <Select
              value={config.currency}
              onValueChange={(val) => {
                const code = val as CurrencyCode;
                const preset = CURRENCIES.find((c) => c.code === code);
                onChangeConfig({
                  ...config,
                  currency: code,
                  vatPolicy: {
                    ...config.vatPolicy,
                    projectVatPct:
                      preset?.vat ?? config.vatPolicy.projectVatPct,
                  },
                });
              }}
            >
              <SelectTrigger className="h-7 px-2.5 text-[11px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {CURRENCIES.map((c) => (
                  <SelectItem key={c.code} value={c.code}>
                    {c.code} · {c.vat}% VAT
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
