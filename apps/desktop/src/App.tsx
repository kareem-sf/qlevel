import { useCallback, useEffect, useState } from "react";
import {
  QlevelBridgeError,
  qlevelClient,
  type AmbiguityPromptPayload,
  type AmbiguityResolutionChoice,
  type DocumentRoleDto,
  type LevelingProgressEvent,
  type LevelingSessionConfig,
  type LevelingSessionResultDto,
  type ModelInfo,
  type ProviderAuthStatusDto,
  type ProviderKind,
  type StagedInputDocumentDto,
  type WorkbookExportOutcomeDto,
} from "@qlevel/bindings";
import {
  Badge,
  Button,
  ProgressBar,
  SegmentedToggle,
  SurfaceCard,
} from "@qlevel/design-system";
import { AdvancedOptionsPanel } from "./components/AdvancedOptionsPanel";
import { AmbiguityDialog } from "./components/AmbiguityDialog";
import { AuthModal } from "./components/AuthModal";
import { DropzoneWorkspace } from "./components/DropzoneWorkspace";
import { ResultView } from "./components/ResultView";
import { DICTIONARIES, type UiLanguage } from "./i18n";

function createInitialConfig(lang: UiLanguage): LevelingSessionConfig {
  return {
    sessionId: `SES-${Date.now()}`,
    ai: {
      provider: null,
      modelId: null,
      effort: "auto",
    },
    currency: "SAR",
    vatPolicy: {
      projectVatPct: "15",
      defaultVendorMode: "exclusive",
    },
    layoutMode: "single_master_sheet",
    mergedItemMode: "interactive_review",
    peerPlugStrategy: "lowest_compliant",
    uiLocale: lang,
    sheetDirection: "auto",
    approvedVendors: [],
    customRules: [],
  };
}

function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      const res = typeof reader.result === "string" ? reader.result : "";
      const comma = res.indexOf(",");
      resolve(comma >= 0 ? res.slice(comma + 1) : res);
    };
    reader.onerror = () => reject(new Error("Failed reading pasted image"));
    reader.readAsDataURL(blob);
  });
}

export function App() {
  const [uiLang, setUiLang] = useState<UiLanguage>("ar");
  const dict = DICTIONARIES[uiLang];

  const [config, setConfig] = useState<LevelingSessionConfig>(() =>
    createInitialConfig("ar")
  );
  const [avlText, setAvlText] = useState<string>("");
  const [customRulesText, setCustomRulesText] = useState<string>("");
  const [advancedOpen, setAdvancedOpen] = useState<boolean>(false);

  const [authModalOpen, setAuthModalOpen] = useState<boolean>(false);
  const [statuses, setStatuses] = useState<ProviderAuthStatusDto[]>([]);
  const [liveModels, setLiveModels] = useState<ModelInfo[]>([]);
  const [loadingModels, setLoadingModels] = useState<boolean>(false);

  const [documents, setDocuments] = useState<StagedInputDocumentDto[]>([]);
  const [isDragging, setIsDragging] = useState<boolean>(false);

  const [busy, setBusy] = useState<boolean>(false);
  const [progress, setProgress] = useState<LevelingProgressEvent | null>(null);
  const [ambiguityPrompt, setAmbiguityPrompt] =
    useState<AmbiguityPromptPayload | null>(null);
  const [result, setResult] = useState<LevelingSessionResultDto | null>(null);
  const [exportOutcome, setExportOutcome] =
    useState<WorkbookExportOutcomeDto | null>(null);
  const [errorBanner, setErrorBanner] = useState<string | null>(null);
  const [manageUsageUrl, setManageUsageUrl] = useState<string | null>(null);

  useEffect(() => {
    document.documentElement.lang = uiLang;
    document.documentElement.dir = uiLang === "ar" ? "rtl" : "ltr";
  }, [uiLang]);

  const refreshStatuses = useCallback(async () => {
    try {
      const list = await qlevelClient.listProviderStatuses();
      setStatuses(list);
    } catch {
      // Headless browser preview fallback
    }
  }, []);

  const refreshLiveModels = useCallback(async () => {
    setLoadingModels(true);
    try {
      const models = await qlevelClient.fetchMultimodalModels(
        config.ai.provider
      );
      setLiveModels(models);
    } catch (err: unknown) {
      if (err instanceof QlevelBridgeError) {
        setErrorBanner(err.payload.message);
      }
    } finally {
      setLoadingModels(false);
    }
  }, [config.ai.provider]);

  useEffect(() => {
    void refreshStatuses();
  }, [refreshStatuses]);

  // Subscribe to Tauri progress, ambiguity prompts, and native drag-and-drop
  useEffect(() => {
    let active = true;
    const unsubs: Array<() => void> = [];

    void (async () => {
      const u1 = await qlevelClient.onProgress((ev) => {
        if (active) {
          setProgress(ev);
        }
      });
      const u2 = await qlevelClient.onAmbiguityPrompt((ev) => {
        if (active) {
          setAmbiguityPrompt(ev);
        }
      });
      const u3 = await qlevelClient.onNativeFileDrop((ev) => {
        if (!active) {
          return;
        }
        if (ev.kind === "hover") {
          setIsDragging(true);
        } else if (ev.kind === "cancel") {
          setIsDragging(false);
        } else if (ev.kind === "drop" && ev.paths.length > 0) {
          setIsDragging(false);
          void qlevelClient
            .triageDroppedFiles(ev.paths)
            .then((staged) => {
              setDocuments((prev) => [...prev, ...staged]);
              setErrorBanner(null);
            })
            .catch((err: unknown) => {
              if (err instanceof QlevelBridgeError) {
                setErrorBanner(err.payload.message);
              }
            });
        }
      });
      unsubs.push(u1, u2, u3);
    })();

    return () => {
      active = false;
      for (const fn of unsubs) {
        fn();
      }
    };
  }, []);

  // Global Ctrl+V clipboard listener for instant Excel/email table & WhatsApp image paste
  useEffect(() => {
    const onPaste = (e: ClipboardEvent) => {
      const target = e.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable)
      ) {
        return;
      }
      const cd = e.clipboardData;
      if (!cd) {
        return;
      }

      const items = Array.from(cd.items);
      const imageItem = items.find((it) => it.type.startsWith("image/"));
      if (imageItem) {
        const file = imageItem.getAsFile();
        if (file) {
          e.preventDefault();
          void blobToBase64(file)
            .then((b64) =>
              qlevelClient.triagePastedClipboard({
                html: null,
                plainText: null,
                imageBase64: b64,
                mimeType: file.type,
              })
            )
            .then((doc) => setDocuments((prev) => [...prev, doc]))
            .catch((err: unknown) => {
              if (err instanceof QlevelBridgeError) {
                setErrorBanner(err.payload.message);
              }
            });
        }
        return;
      }

      const html = cd.getData("text/html") || null;
      const plainText = cd.getData("text/plain") || null;
      if ((html && html.includes("<tr")) || (plainText && plainText.includes("\t"))) {
        e.preventDefault();
        void qlevelClient
          .triagePastedClipboard({
            html,
            plainText,
            imageBase64: null,
            mimeType: "text/plain",
          })
          .then((doc) => setDocuments((prev) => [...prev, doc]))
          .catch((err: unknown) => {
            if (err instanceof QlevelBridgeError) {
              setErrorBanner(err.payload.message);
            }
          });
      }
    };

    window.addEventListener("paste", onPaste);
    return () => window.removeEventListener("paste", onPaste);
  }, []);

  const anyConnected = statuses.some((s) => s.connected);
  const hasVendorQuote = documents.some((d) => d.role === "vendor_quote");

  const handleBrowseFiles = async () => {
    try {
      const picked = await qlevelClient.pickInputFiles();
      if (picked.length > 0) {
        const staged = await qlevelClient.triageDroppedFiles(picked);
        setDocuments((prev) => [...prev, ...staged]);
        setErrorBanner(null);
      }
    } catch (err: unknown) {
      if (err instanceof QlevelBridgeError) {
        setErrorBanner(err.payload.message);
      }
    }
  };

  const handleStartComparison = async () => {
    if (!anyConnected) {
      setAuthModalOpen(true);
      return;
    }
    setBusy(true);
    setErrorBanner(null);
    setManageUsageUrl(null);
    setExportOutcome(null);

    const sessionId = `SES-${Date.now()}`;
    const preparedConfig: LevelingSessionConfig = {
      ...config,
      sessionId,
      uiLocale: uiLang,
      approvedVendors: avlText
        .split(",")
        .map((s) => s.trim())
        .filter((s) => s.length > 0),
      customRules: customRulesText
        .split("\n")
        .map((s) => s.trim())
        .filter((s) => s.length > 0),
    };
    setConfig(preparedConfig);

    try {
      const sessionResult = await qlevelClient.startLevelingSession({
        documents,
        config: preparedConfig,
      });
      setResult(sessionResult);
    } catch (err: unknown) {
      if (err instanceof QlevelBridgeError) {
        setErrorBanner(err.payload.message);
        setManageUsageUrl(err.payload.manageUsageUrl);
      } else if (err instanceof Error) {
        setErrorBanner(err.message);
      }
    } finally {
      setBusy(false);
      setProgress(null);
      setAmbiguityPrompt(null);
    }
  };

  const handleSaveWorkbook = async (useFallbackIfLocked: boolean) => {
    if (!result) {
      return;
    }
    setBusy(true);
    try {
      let targetPath = result.suggestedOutputPath;
      if (!useFallbackIfLocked) {
        const chosen = await qlevelClient.pickSaveWorkbookPath(
          result.suggestedOutputPath
        );
        if (!chosen) {
          setBusy(false);
          return;
        }
        targetPath = chosen;
      }
      const outcome = await qlevelClient.exportWorkbook({
        sessionId: result.sessionId,
        targetPath,
        useFallbackIfLocked,
      });
      setExportOutcome(outcome);
    } catch (err: unknown) {
      if (err instanceof QlevelBridgeError) {
        setErrorBanner(err.payload.message);
      }
    } finally {
      setBusy(false);
    }
  };

  const handleResolveAmbiguity = async (choice: AmbiguityResolutionChoice) => {
    if (!ambiguityPrompt) {
      return;
    }
    const current = ambiguityPrompt;
    setAmbiguityPrompt(null);
    await qlevelClient.resolveAmbiguityPrompt({
      sessionId: current.sessionId,
      groupId: current.group.groupId,
      choice,
      reviewerNote: null,
    });
  };

  const handleSaveApiKey = async (
    provider: ProviderKind,
    apiKey: string,
    ackGeminiPrivacy: boolean
  ) => {
    setBusy(true);
    setErrorBanner(null);
    try {
      await qlevelClient.saveProviderApiKey({
        provider,
        apiKey,
        acknowledgeGeminiFreeTierPrivacy: ackGeminiPrivacy,
      });
      await refreshStatuses();
      await refreshLiveModels();
    } catch (err: unknown) {
      if (err instanceof QlevelBridgeError) {
        setErrorBanner(err.payload.message);
      }
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="min-h-screen bg-slate-50">
      <header className="border-b border-slate-200 bg-white">
        <div className="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-4 px-6 py-3.5">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-teal-700 text-sm font-bold text-white">
              qL
            </div>
            <div>
              <h1 className="text-base font-bold text-slate-900">
                {dict.appName}
              </h1>
              <p className="text-xs text-slate-500">{dict.appTagline}</p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <SegmentedToggle<UiLanguage>
              ariaLabel="Language"
              value={uiLang}
              onChange={setUiLang}
              options={[
                { value: "ar", label: "العربية" },
                { value: "en", label: "EN" },
              ]}
            />
            <Button
              variant="secondary"
              onClick={() => setAuthModalOpen(true)}
            >
              <span>{dict.connectAiBtn}</span>
              <Badge tone={anyConnected ? "success" : "warning"}>
                {anyConnected ? dict.connectedBadge : dict.notConnectedBadge}
              </Badge>
            </Button>
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-5xl space-y-5 px-6 py-6">
        {errorBanner && (
          <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-xs font-medium text-red-900">
            <span>{errorBanner}</span>
            {manageUsageUrl && (
              <Button
                variant="secondary"
                onClick={() => void qlevelClient.openExternalUrl(manageUsageUrl)}
              >
                Manage Usage
              </Button>
            )}
          </div>
        )}

        {result ? (
          <ResultView
            dict={dict}
            result={result}
            exportOutcome={exportOutcome}
            busy={busy}
            onSaveWorkbook={(fallback) => void handleSaveWorkbook(fallback)}
            onOpenInExcel={(p) => void qlevelClient.openWorkbookInExcel(p)}
            onRevealFolder={(p) => void qlevelClient.revealWorkbookInFolder(p)}
            onResetSession={() => {
              setResult(null);
              setExportOutcome(null);
            }}
          />
        ) : (
          <>
            <DropzoneWorkspace
              dict={dict}
              documents={documents}
              isDragging={isDragging}
              busy={busy}
              onBrowseFiles={() => void handleBrowseFiles()}
              onChangeRole={(id: string, role: DocumentRoleDto) =>
                setDocuments((prev) =>
                  prev.map((d) => (d.id === id ? { ...d, role } : d))
                )
              }
              onChangeVendorName={(id: string, vendorName: string) =>
                setDocuments((prev) =>
                  prev.map((d) => (d.id === id ? { ...d, vendorName } : d))
                )
              }
              onRemoveDocument={(id: string) =>
                setDocuments((prev) => prev.filter((d) => d.id !== id))
              }
            />

            <AdvancedOptionsPanel
              dict={dict}
              open={advancedOpen}
              onToggleOpen={() => {
                const next = !advancedOpen;
                setAdvancedOpen(next);
                if (next && liveModels.length === 0 && anyConnected) {
                  void refreshLiveModels();
                }
              }}
              config={config}
              liveModels={liveModels}
              loadingModels={loadingModels}
              avlText={avlText}
              customRulesText={customRulesText}
              onChangeConfig={setConfig}
              onChangeAvlText={setAvlText}
              onChangeCustomRulesText={setCustomRulesText}
              onRefreshLiveModels={() => void refreshLiveModels()}
              onPickCustomRulesFile={() => {
                void qlevelClient.pickCustomRulesFile().then((path) => {
                  if (path) {
                    setCustomRulesText((prev) =>
                      prev ? `${prev}\n@${path}` : `@${path}`
                    );
                  }
                });
              }}
            />

            {busy && progress ? (
              <SurfaceCard className="space-y-3">
                <ProgressBar
                  percent={progress.percent}
                  label={`${progress.stage}${
                    progress.activeVendorName
                      ? ` - ${progress.activeVendorName}`
                      : ""
                  }`}
                  sublabel={
                    progress.checkpointHit
                      ? "SQLite WAL checkpoint resumed"
                      : undefined
                  }
                />
                <div className="flex justify-end">
                  <Button
                    variant="danger"
                    onClick={() =>
                      void qlevelClient.cancelLevelingSession(config.sessionId)
                    }
                  >
                    {dict.cancelRunBtn}
                  </Button>
                </div>
              </SurfaceCard>
            ) : (
              <div className="flex justify-end">
                <Button
                  variant="primary"
                  disabled={busy || !hasVendorQuote}
                  onClick={() => void handleStartComparison()}
                  className="px-6 py-2.5 text-base"
                >
                  {dict.compareAndExportBtn}
                </Button>
              </div>
            )}
          </>
        )}
      </main>

      <AuthModal
        open={authModalOpen}
        dict={dict}
        statuses={statuses}
        busy={busy}
        errorMessage={errorBanner}
        onClose={() => setAuthModalOpen(false)}
        onSignInChatGpt={() => {
          setBusy(true);
          setErrorBanner(null);
          void qlevelClient
            .startOpenAiChatGptSignIn()
            .then(() => refreshStatuses())
            .then(() => refreshLiveModels())
            .catch((err: unknown) => {
              if (err instanceof QlevelBridgeError) {
                setErrorBanner(err.payload.message);
              }
            })
            .finally(() => setBusy(false));
        }}
        onSaveApiKey={(p, k, ack) => void handleSaveApiKey(p, k, ack)}
        onDisconnect={(p) => {
          void qlevelClient
            .disconnectProvider(p)
            .then(() => refreshStatuses());
        }}
      />

      <AmbiguityDialog
        prompt={ambiguityPrompt}
        dict={dict}
        onResolve={(choice) => void handleResolveAmbiguity(choice)}
      />
    </div>
  );
}
