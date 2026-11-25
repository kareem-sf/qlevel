import type {
  CurrencyCode,
  EffortLevel,
  LevelingSessionConfig,
  MergedItemMode,
  ModelInfo,
  PeerPlugStrategy,
  ProviderKind,
  SheetDirection,
  WorkbookLayoutMode,
} from "@qlevel/bindings";
import { Button, CollapsibleSection, SegmentedToggle } from "@qlevel/design-system";
import type { Dictionary } from "../i18n";

export interface AdvancedOptionsPanelProps {
  dict: Dictionary;
  open: boolean;
  onToggleOpen: () => void;
  config: LevelingSessionConfig;
  liveModels: readonly ModelInfo[];
  loadingModels: boolean;
  avlText: string;
  customRulesText: string;
  onChangeConfig: (next: LevelingSessionConfig) => void;
  onChangeAvlText: (val: string) => void;
  onChangeCustomRulesText: (val: string) => void;
  onRefreshLiveModels: () => void;
  onPickCustomRulesFile: () => void;
}

const CURRENCIES: readonly { code: CurrencyCode; defaultVat: string }[] = [
  { code: "SAR", defaultVat: "15" },
  { code: "EGP", defaultVat: "14" },
  { code: "AED", defaultVat: "5" },
  { code: "QAR", defaultVat: "0" },
  { code: "KWD", defaultVat: "0" },
  { code: "BHD", defaultVat: "5" },
  { code: "OMR", defaultVat: "5" },
  { code: "USD", defaultVat: "0" },
  { code: "EUR", defaultVat: "0" },
];

export function AdvancedOptionsPanel({
  dict,
  open,
  onToggleOpen,
  config,
  liveModels,
  loadingModels,
  avlText,
  customRulesText,
  onChangeConfig,
  onChangeAvlText,
  onChangeCustomRulesText,
  onRefreshLiveModels,
  onPickCustomRulesFile,
}: AdvancedOptionsPanelProps) {
  const activeModelLabel =
    config.ai.modelId === null || config.ai.modelId === "auto"
      ? dict.modelAuto
      : config.ai.modelId;
  const layoutSummary =
    config.layoutMode === "single_master_sheet"
      ? dict.layoutSingle
      : dict.layoutDual;
  const summaryText = `${activeModelLabel} | ${config.currency} (${config.vatPolicy.projectVatPct}% VAT) | ${layoutSummary}`;

  return (
    <CollapsibleSection
      title={dict.advancedTitle}
      summaryText={summaryText}
      open={open}
      onToggle={onToggleOpen}
    >
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <div className="space-y-1.5">
          <label className="block text-xs font-semibold text-slate-700">
            {dict.providerLabel}
          </label>
          <select
            value={config.ai.provider ?? "auto"}
            onChange={(e) => {
              const v = e.target.value;
              const nextProvider: ProviderKind | null =
                v === "open_ai" || v === "gemini" || v === "claude" ? v : null;
              onChangeConfig({
                ...config,
                ai: { ...config.ai, provider: nextProvider, modelId: null },
              });
            }}
            className="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-xs text-slate-900"
          >
            <option value="auto">{dict.providerAuto}</option>
            <option value="open_ai">OpenAI</option>
            <option value="gemini">Google Gemini</option>
            <option value="claude">Anthropic Claude</option>
          </select>
        </div>

        <div className="space-y-1.5">
          <div className="flex items-center justify-between">
            <label className="block text-xs font-semibold text-slate-700">
              {dict.modelLabel}
            </label>
            <button
              type="button"
              disabled={loadingModels}
              onClick={onRefreshLiveModels}
              className="text-xs font-semibold text-teal-700 hover:underline"
            >
              {dict.refreshModelsBtn}
            </button>
          </div>
          <select
            value={config.ai.modelId ?? "auto"}
            onChange={(e) => {
              const val = e.target.value;
              onChangeConfig({
                ...config,
                ai: { ...config.ai, modelId: val === "auto" ? null : val },
              });
            }}
            className="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-xs text-slate-900"
          >
            <option value="auto">{dict.modelAuto}</option>
            {liveModels.map((m) => (
              <option key={`${m.provider}-${m.id}`} value={m.id}>
                {m.displayName} ({m.id})
              </option>
            ))}
          </select>
        </div>

        <div className="space-y-1.5">
          <label className="block text-xs font-semibold text-slate-700">
            {dict.effortLabel}
          </label>
          <SegmentedToggle<EffortLevel>
            ariaLabel={dict.effortLabel}
            value={config.ai.effort}
            onChange={(effort) =>
              onChangeConfig({ ...config, ai: { ...config.ai, effort } })
            }
            options={[
              { value: "auto", label: dict.effortAuto },
              { value: "low", label: dict.effortLow },
              { value: "medium", label: dict.effortMedium },
              { value: "high", label: dict.effortHigh },
            ]}
          />
        </div>

        <div className="space-y-1.5">
          <label className="block text-xs font-semibold text-slate-700">
            {dict.layoutLabel}
          </label>
          <SegmentedToggle<WorkbookLayoutMode>
            ariaLabel={dict.layoutLabel}
            value={config.layoutMode}
            onChange={(layoutMode) => onChangeConfig({ ...config, layoutMode })}
            options={[
              { value: "single_master_sheet", label: dict.layoutSingle },
              { value: "dual_tab", label: dict.layoutDual },
            ]}
          />
        </div>

        <div className="space-y-1.5">
          <label className="block text-xs font-semibold text-slate-700">
            {dict.mergedModeLabel}
          </label>
          <SegmentedToggle<MergedItemMode>
            ariaLabel={dict.mergedModeLabel}
            value={config.mergedItemMode}
            onChange={(mergedItemMode) =>
              onChangeConfig({ ...config, mergedItemMode })
            }
            options={[
              { value: "interactive_review", label: dict.mergedInteractive },
              { value: "yellow_highlight_auto", label: dict.mergedYellow },
            ]}
          />
        </div>

        <div className="space-y-1.5">
          <label className="block text-xs font-semibold text-slate-700">
            {dict.plugBasisLabel}
          </label>
          <select
            value={config.peerPlugStrategy}
            onChange={(e) =>
              onChangeConfig({
                ...config,
                peerPlugStrategy: e.target.value as PeerPlugStrategy,
              })
            }
            className="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-xs text-slate-900"
          >
            <option value="lowest_compliant">{dict.plugLowest}</option>
            <option value="average_compliant">{dict.plugAverage}</option>
            <option value="highest_compliant">{dict.plugHighest}</option>
            <option value="pte_estimate">{dict.plugPte}</option>
          </select>
        </div>

        <div className="grid grid-cols-3 gap-2 md:col-span-2">
          <div className="space-y-1">
            <label className="block text-xs font-semibold text-slate-700">
              {dict.currencyLabel}
            </label>
            <select
              value={config.currency}
              onChange={(e) => {
                const nextCode = e.target.value as CurrencyCode;
                const preset = CURRENCIES.find((c) => c.code === nextCode);
                onChangeConfig({
                  ...config,
                  currency: nextCode,
                  vatPolicy: {
                    ...config.vatPolicy,
                    projectVatPct:
                      preset?.defaultVat ?? config.vatPolicy.projectVatPct,
                  },
                });
              }}
              className="w-full rounded-lg border border-slate-300 bg-white px-2.5 py-2 text-xs text-slate-900"
            >
              {CURRENCIES.map((c) => (
                <option key={c.code} value={c.code}>
                  {c.code}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-1">
            <label className="block text-xs font-semibold text-slate-700">
              {dict.vatLabel}
            </label>
            <input
              type="text"
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
              className="w-full rounded-lg border border-slate-300 bg-white px-2.5 py-2 text-xs text-slate-900 tabular-nums"
            />
          </div>

          <div className="space-y-1">
            <label className="block text-xs font-semibold text-slate-700">
              {dict.sheetDirLabel}
            </label>
            <select
              value={config.sheetDirection}
              onChange={(e) =>
                onChangeConfig({
                  ...config,
                  sheetDirection: e.target.value as SheetDirection,
                })
              }
              className="w-full rounded-lg border border-slate-300 bg-white px-2.5 py-2 text-xs text-slate-900"
            >
              <option value="auto">Auto</option>
              <option value="rtl">RTL (العربية)</option>
              <option value="ltr">LTR (English)</option>
            </select>
          </div>
        </div>

        <div className="space-y-1.5 md:col-span-2">
          <label className="block text-xs font-semibold text-slate-700">
            {dict.avlLabel}
          </label>
          <input
            type="text"
            value={avlText}
            onChange={(e) => onChangeAvlText(e.target.value)}
            placeholder={dict.avlPlaceholder}
            className="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-xs text-slate-900"
          />
        </div>

        <div className="space-y-1.5 md:col-span-2">
          <div className="flex items-center justify-between">
            <label className="block text-xs font-semibold text-slate-700">
              {dict.customRulesLabel}
            </label>
            <Button
              variant="ghost"
              onClick={onPickCustomRulesFile}
              className="px-2 py-1 text-xs"
            >
              {dict.loadRulesFileBtn}
            </Button>
          </div>
          <textarea
            rows={3}
            value={customRulesText}
            onChange={(e) => onChangeCustomRulesText(e.target.value)}
            placeholder={dict.customRulesPlaceholder}
            className="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-xs text-slate-900"
          />
        </div>
      </div>
    </CollapsibleSection>
  );
}
