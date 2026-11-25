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
  CommandPalette,
  QlevelToaster,
  type CommandPaletteAction,
} from "@qlevel/design-system";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { AgenticIntakeView } from "./components/AgenticIntakeView";
import { EqualizedResultsView } from "./components/EqualizedResultsView";
import { IconRail, type WorkspaceScreen } from "./components/IconRail";
import { LiveAgenticSplitView } from "./components/LiveAgenticSplitView";
import {
  SettingsCommandPopover,
  type SettingsTabId,
} from "./components/SettingsCommandPopover";
import { DICTIONARIES, type UiLanguage } from "./i18n";
import { rasterizePdfPagesInMemory } from "./pdfRenderer";

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
    reader.onerror = () => reject(new Error("Clipboard read failed"));
    reader.readAsDataURL(blob);
  });
}

export function App() {
  const [uiLang, setUiLang] = useState<UiLanguage>("en");
  const [dark, setDark] = useState<boolean>(false);
  const dict = DICTIONARIES[uiLang];

  const [activeScreen, setActiveScreen] = useState<WorkspaceScreen>("intake");
  const [commandOpen, setCommandOpen] = useState<boolean>(false);
  const [settingsOpen, setSettingsOpen] = useState<boolean>(false);
  const [settingsTab, setSettingsTab] = useState<SettingsTabId>("providers");

  const [config, setConfig] = useState<LevelingSessionConfig>(() =>
    createInitialConfig("en")
  );
  const [avlText, setAvlText] = useState<string>("");
  const [customRulesText, setCustomRulesText] = useState<string>("");

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

  useEffect(() => {
    document.documentElement.lang = uiLang;
    document.documentElement.dir = uiLang === "ar" ? "rtl" : "ltr";
  }, [uiLang]);

  useEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
  }, [dark]);

  // Global Cmd+K / Ctrl+K command menu shortcut
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setCommandOpen((prev) => !prev);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const refreshStatuses = useCallback(async () => {
    try {
      const list = await qlevelClient.listProviderStatuses();
      setStatuses(list);
    } catch {
      // Browser preview fallback
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
        toast.error(err.payload.message);
      }
    } finally {
      setLoadingModels(false);
    }
  }, [config.ai.provider]);

  useEffect(() => {
    void refreshStatuses();
  }, [refreshStatuses]);

  // Subscribe to Tauri events (progress, inline A2UI ambiguity prompt, PDF rasterizer, native drop)
  useEffect(() => {
    let mounted = true;
    const unsubs: Array<() => void> = [];

    void (async () => {
      try {
        const u1 = await qlevelClient.onProgress((ev) => {
          if (mounted) {
            setProgress(ev);
          }
        });
        const u2 = await qlevelClient.onAmbiguityPrompt((ev) => {
          if (mounted) {
            setAmbiguityPrompt(ev);
          }
        });
        const u3 = await qlevelClient.onPdfRasterRequest((ev) => {
          if (!mounted) return;
          void qlevelClient
            .readPdfBytes(ev.filePath)
            .then((bytes) =>
              rasterizePdfPagesInMemory(
                new Uint8Array(bytes),
                ev.pageNumbers,
                ev.targetDpi
              )
            )
            .then((pages) =>
              qlevelClient.submitRenderedPdfPages({
                documentId: ev.documentId,
                pages,
              })
            )
            .catch(() => undefined);
        });
        const u4 = await qlevelClient.onNativeFileDrop((ev) => {
          if (!mounted) return;
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
              })
              .catch((err: unknown) => {
                if (err instanceof QlevelBridgeError) {
                  toast.error(err.payload.message);
                }
              });
          }
        });
        unsubs.push(u1, u2, u3, u4);
      } catch {
        // Non-Tauri web preview environment
      }
    })();

    return () => {
      mounted = false;
      for (const fn of unsubs) {
        fn();
      }
    };
  }, []);

  // Global Ctrl+V clipboard listener for instant table & screenshot capture
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
      if (!cd) return;

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
            .then((doc) => {
              setDocuments((prev) => [...prev, doc]);
              toast.success(doc.fileName);
            })
            .catch((err: unknown) => {
              if (err instanceof QlevelBridgeError) {
                toast.error(err.payload.message);
              }
            });
        }
        return;
      }

      const html = cd.getData("text/html") || null;
      const plainText = cd.getData("text/plain") || null;
      if (
        (html && html.includes("<tr")) ||
        (plainText && plainText.includes("\t"))
      ) {
        e.preventDefault();
        void qlevelClient
          .triagePastedClipboard({
            html,
            plainText,
            imageBase64: null,
            mimeType: "text/plain",
          })
          .then((doc) => {
            setDocuments((prev) => [...prev, doc]);
            toast.success(doc.fileName);
          })
          .catch((err: unknown) => {
            if (err instanceof QlevelBridgeError) {
              toast.error(err.payload.message);
            }
          });
      }
    };

    window.addEventListener("paste", onPaste);
    return () => window.removeEventListener("paste", onPaste);
  }, []);

  const anyConnected = statuses.some((s) => s.connected);

  const openSettingsAt = useCallback(
    (tab: SettingsTabId = "providers") => {
      setSettingsTab(tab);
      setSettingsOpen(true);
      if (liveModels.length === 0 && anyConnected) {
        void refreshLiveModels();
      }
    },
    [anyConnected, liveModels.length, refreshLiveModels]
  );

  const handleBrowseFiles = async () => {
    try {
      const picked = await qlevelClient.pickInputFiles();
      if (picked.length > 0) {
        const staged = await qlevelClient.triageDroppedFiles(picked);
        setDocuments((prev) => [...prev, ...staged]);
      }
    } catch (err: unknown) {
      if (err instanceof QlevelBridgeError) {
        toast.error(err.payload.message);
      }
    }
  };

  const handleStartLeveling = async () => {
    if (!anyConnected) {
      openSettingsAt("providers");
      return;
    }
    setBusy(true);
    setExportOutcome(null);
    setActiveScreen("live");

    const sessionId = `SES-${Date.now()}`;
    const prepared: LevelingSessionConfig = {
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
    setConfig(prepared);

    try {
      const sessionResult = await qlevelClient.startLevelingSession({
        documents,
        config: prepared,
      });
      setResult(sessionResult);
      setActiveScreen("results");
    } catch (err: unknown) {
      setActiveScreen("intake");
      if (err instanceof QlevelBridgeError) {
        toast.error(err.payload.message, {
          action: err.payload.manageUsageUrl
            ? {
                label: "Manage",
                onClick: () =>
                  void qlevelClient.openExternalUrl(
                    err.payload.manageUsageUrl ?? ""
                  ),
              }
            : undefined,
        });
      } else if (err instanceof Error) {
        toast.error(err.message);
      }
    } finally {
      setBusy(false);
      setProgress(null);
      setAmbiguityPrompt(null);
    }
  };

  const handleSaveWorkbook = async (useFallbackIfLocked: boolean) => {
    if (!result) return;
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
      if (outcome.status === "saved") {
        toast.success(outcome.outputPath);
      }
    } catch (err: unknown) {
      if (err instanceof QlevelBridgeError) {
        toast.error(err.payload.message);
      }
    } finally {
      setBusy(false);
    }
  };

  const handleResolveAmbiguity = async (choice: AmbiguityResolutionChoice) => {
    if (!ambiguityPrompt) return;
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
    try {
      await qlevelClient.saveProviderApiKey({
        provider,
        apiKey,
        acknowledgeGeminiFreeTierPrivacy: ackGeminiPrivacy,
      });
      await refreshStatuses();
      await refreshLiveModels();
      toast.success(dict.connectedLabel);
    } catch (err: unknown) {
      if (err instanceof QlevelBridgeError) {
        toast.error(err.payload.message);
      }
    } finally {
      setBusy(false);
    }
  };

  const commandActions: readonly CommandPaletteAction[] = useMemo(
    () => [
      {
        id: "go-intake",
        group: "Workspace",
        label: dict.navIntake,
        shortcut: "1",
        onSelect: () => setActiveScreen("intake"),
      },
      ...(result
        ? [
            {
              id: "go-results",
              group: "Workspace",
              label: dict.navResults,
              shortcut: "3",
              onSelect: () => setActiveScreen("results"),
            },
          ]
        : []),
      {
        id: "open-providers",
        group: "Settings",
        label: dict.settingsTabProviders,
        onSelect: () => openSettingsAt("providers"),
      },
      {
        id: "open-rules",
        group: "Settings",
        label: dict.settingsTabRules,
        onSelect: () => openSettingsAt("rules"),
      },
      {
        id: "toggle-lang",
        group: "Preferences",
        label: uiLang === "en" ? "Switch to Arabic (RTL)" : "Switch to English (LTR)",
        onSelect: () => setUiLang((prev) => (prev === "en" ? "ar" : "en")),
      },
      {
        id: "toggle-theme",
        group: "Preferences",
        label: dict.navTheme,
        onSelect: () => setDark((prev) => !prev),
      },
    ],
    [dict, openSettingsAt, result, uiLang]
  );

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[var(--background)] text-[var(--foreground)]">
      <IconRail
        activeScreen={activeScreen}
        hasResults={result !== null}
        isBusy={busy}
        dark={dark}
        dict={dict}
        onSelectScreen={setActiveScreen}
        onOpenCommand={() => setCommandOpen(true)}
        onToggleDark={() => setDark((prev) => !prev)}
        onOpenSettings={() => openSettingsAt("providers")}
      />

      {activeScreen === "intake" && (
        <AgenticIntakeView
          dict={dict}
          uiLang={uiLang}
          config={config}
          documents={documents}
          isDragging={isDragging}
          anyProviderConnected={anyConnected}
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
          onToggleLanguage={() =>
            setUiLang((prev) => (prev === "en" ? "ar" : "en"))
          }
          onToggleLayout={() =>
            setConfig((prev) => ({
              ...prev,
              layoutMode:
                prev.layoutMode === "single_master_sheet"
                  ? "dual_tab"
                  : "single_master_sheet",
            }))
          }
          onOpenSettings={(tab) => openSettingsAt(tab)}
          onStartLeveling={() => void handleStartLeveling()}
        />
      )}

      {activeScreen === "live" && (
        <LiveAgenticSplitView
          dict={dict}
          currency={config.currency}
          documents={documents}
          progress={progress}
          ambiguityPrompt={ambiguityPrompt}
          onResolveAmbiguity={(c) => void handleResolveAmbiguity(c)}
          onCancel={() => {
            void qlevelClient.cancelLevelingSession(config.sessionId);
            setActiveScreen("intake");
          }}
        />
      )}

      {activeScreen === "results" && result && (
        <EqualizedResultsView
          dict={dict}
          result={result}
          exportOutcome={exportOutcome}
          busy={busy}
          onSaveWorkbook={(fallback) => void handleSaveWorkbook(fallback)}
          onOpenInExcel={(p) => void qlevelClient.openWorkbookInExcel(p)}
          onRevealFolder={(p) => void qlevelClient.revealWorkbookInFolder(p)}
        />
      )}

      <SettingsCommandPopover
        open={settingsOpen}
        initialTab={settingsTab}
        dict={dict}
        config={config}
        statuses={statuses}
        liveModels={liveModels}
        loadingModels={loadingModels}
        avlText={avlText}
        customRulesText={customRulesText}
        busy={busy}
        onOpenChange={setSettingsOpen}
        onChangeTab={setSettingsTab}
        onChangeConfig={setConfig}
        onChangeAvlText={setAvlText}
        onChangeCustomRulesText={setCustomRulesText}
        onRefreshModels={() => void refreshLiveModels()}
        onPickRulesFile={() => {
          void qlevelClient.pickCustomRulesFile().then((path) => {
            if (path) {
              setCustomRulesText((prev) =>
                prev ? `${prev}\n@${path}` : `@${path}`
              );
            }
          });
        }}
        onSignInChatGpt={() => {
          setBusy(true);
          void qlevelClient
            .startOpenAiChatGptSignIn()
            .then(() => refreshStatuses())
            .then(() => refreshLiveModels())
            .then(() => toast.success(dict.connectedLabel))
            .catch((err: unknown) => {
              if (err instanceof QlevelBridgeError) {
                toast.error(err.payload.message);
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

      <CommandPalette
        open={commandOpen}
        onOpenChange={setCommandOpen}
        placeholder={dict.commandPlaceholder}
        emptyText={dict.commandEmpty}
        actions={commandActions}
      />

      <QlevelToaster dark={dark} />
    </div>
  );
}
