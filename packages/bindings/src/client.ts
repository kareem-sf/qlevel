import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type {
  AmbiguityDecisionInput,
  AmbiguityPromptPayload,
  BridgeErrorCode,
  BridgeErrorPayload,
  ExportWorkbookInput,
  LevelingProgressEvent,
  LevelingSessionResultDto,
  ModelInfo,
  PdfRasterRequestEvent,
  ProviderAuthStatusDto,
  ProviderKind,
  SaveApiKeyInput,
  StagedInputDocumentDto,
  StartLevelingSessionInput,
  SubmitRenderedPdfPagesInput,
  TriageClipboardInput,
  UpdateCheckStatusDto,
  WorkbookExportOutcomeDto,
} from "./generated";

export interface NativeFileDropPayload {
  kind: "hover" | "drop" | "cancel";
  paths: string[];
}

const KNOWN_ERROR_CODES: ReadonlySet<BridgeErrorCode> = new Set<BridgeErrorCode>([
  "auth_expired",
  "openai_usage_limit_exceeded",
  "gemini_free_tier_blocked",
  "file_password_protected",
  "file_corrupted",
  "excel_file_locked",
  "missing_vendor_quote",
  "cancelled",
  "network_error",
  "internal_error",
]);

function isRecord(val: unknown): val is Record<string, unknown> {
  return typeof val === "object" && val !== null;
}

export class QlevelBridgeError extends Error {
  public readonly payload: BridgeErrorPayload;

  public constructor(payload: BridgeErrorPayload) {
    super(payload.message);
    this.name = "QlevelBridgeError";
    this.payload = payload;
  }
}

export function normalizeBridgeError(raw: unknown): QlevelBridgeError {
  if (raw instanceof QlevelBridgeError) {
    return raw;
  }
  if (isRecord(raw)) {
    const rawCode = raw["code"];
    const code: BridgeErrorCode =
      typeof rawCode === "string" && KNOWN_ERROR_CODES.has(rawCode as BridgeErrorCode)
        ? (rawCode as BridgeErrorCode)
        : "internal_error";
    const message =
      typeof raw["message"] === "string" && raw["message"].length > 0
        ? raw["message"]
        : "Unexpected desktop bridge error";
    return new QlevelBridgeError({
      code,
      message,
      affectedFileName:
        typeof raw["affectedFileName"] === "string" ? raw["affectedFileName"] : null,
      manageUsageUrl:
        typeof raw["manageUsageUrl"] === "string" ? raw["manageUsageUrl"] : null,
      suggestedFallbackPath:
        typeof raw["suggestedFallbackPath"] === "string"
          ? raw["suggestedFallbackPath"]
          : null,
    });
  }
  if (raw instanceof Error) {
    return new QlevelBridgeError({
      code: "internal_error",
      message: raw.message,
      affectedFileName: null,
      manageUsageUrl: null,
      suggestedFallbackPath: null,
    });
  }
  return new QlevelBridgeError({
    code: "internal_error",
    message: String(raw),
    affectedFileName: null,
    manageUsageUrl: null,
    suggestedFallbackPath: null,
  });
}

function hasTauriInternals(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  const winObj = window as unknown as Record<string, unknown>;
  return isRecord(winObj["__TAURI_INTERNALS__"]);
}

async function callCommand<T>(
  cmd: string,
  args?: Record<string, unknown> | undefined
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (err: unknown) {
    throw normalizeBridgeError(err);
  }
}

export class QlevelClient {
  public listProviderStatuses(): Promise<ProviderAuthStatusDto[]> {
    return callCommand<ProviderAuthStatusDto[]>("qlevel_list_provider_statuses");
  }

  public startOpenAiChatGptSignIn(): Promise<ProviderAuthStatusDto> {
    return callCommand<ProviderAuthStatusDto>("qlevel_start_openai_chatgpt_signin");
  }

  public saveProviderApiKey(input: SaveApiKeyInput): Promise<ProviderAuthStatusDto> {
    return callCommand<ProviderAuthStatusDto>("qlevel_save_provider_api_key", { input });
  }

  public disconnectProvider(provider: ProviderKind): Promise<ProviderAuthStatusDto> {
    return callCommand<ProviderAuthStatusDto>("qlevel_disconnect_provider", { provider });
  }

  public fetchMultimodalModels(provider: ProviderKind | null): Promise<ModelInfo[]> {
    return callCommand<ModelInfo[]>("qlevel_fetch_multimodal_models", { provider });
  }

  public pickInputFiles(): Promise<string[]> {
    return callCommand<string[]>("qlevel_pick_input_files");
  }

  public pickCustomRulesFile(): Promise<string | null> {
    return callCommand<string | null>("qlevel_pick_custom_rules_file");
  }

  public pickSaveWorkbookPath(suggestedPath: string): Promise<string | null> {
    return callCommand<string | null>("qlevel_pick_save_workbook_path", { suggestedPath });
  }

  public readPdfBytes(filePath: string): Promise<number[]> {
    return callCommand<number[]>("qlevel_read_pdf_bytes", { filePath });
  }

  public triageDroppedFiles(filePaths: string[]): Promise<StagedInputDocumentDto[]> {
    return callCommand<StagedInputDocumentDto[]>("qlevel_triage_dropped_files", {
      filePaths,
    });
  }

  public triagePastedClipboard(
    input: TriageClipboardInput
  ): Promise<StagedInputDocumentDto> {
    return callCommand<StagedInputDocumentDto>("qlevel_triage_pasted_clipboard", {
      input,
    });
  }

  public submitRenderedPdfPages(input: SubmitRenderedPdfPagesInput): Promise<void> {
    return callCommand<void>("qlevel_submit_rendered_pdf_pages", { input });
  }

  public startLevelingSession(
    input: StartLevelingSessionInput
  ): Promise<LevelingSessionResultDto> {
    return callCommand<LevelingSessionResultDto>("qlevel_start_leveling_session", {
      input,
    });
  }

  public cancelLevelingSession(sessionId: string): Promise<void> {
    return callCommand<void>("qlevel_cancel_leveling_session", { sessionId });
  }

  public resolveAmbiguityPrompt(decision: AmbiguityDecisionInput): Promise<void> {
    return callCommand<void>("qlevel_resolve_ambiguity_prompt", { decision });
  }

  public exportWorkbook(input: ExportWorkbookInput): Promise<WorkbookExportOutcomeDto> {
    return callCommand<WorkbookExportOutcomeDto>("qlevel_export_workbook", { input });
  }

  public openWorkbookInExcel(filePath: string): Promise<void> {
    return callCommand<void>("qlevel_open_workbook_in_excel", { filePath });
  }

  public revealWorkbookInFolder(filePath: string): Promise<void> {
    return callCommand<void>("qlevel_reveal_workbook_in_folder", { filePath });
  }

  public openExternalUrl(url: string): Promise<void> {
    return callCommand<void>("qlevel_open_external_url", { url });
  }

  public checkForUpdates(applyPortableUpdate: boolean): Promise<UpdateCheckStatusDto> {
    return callCommand<UpdateCheckStatusDto>("qlevel_check_for_updates", {
      applyPortableUpdate,
    });
  }

  public async onProgress(
    handler: (ev: LevelingProgressEvent) => void
  ): Promise<() => void> {
    if (!hasTauriInternals()) {
      return () => undefined;
    }
    const unlisten = await listen<LevelingProgressEvent>("qlevel://progress", (e) => {
      handler(e.payload);
    });
    return () => unlisten();
  }

  public async onAmbiguityPrompt(
    handler: (ev: AmbiguityPromptPayload) => void
  ): Promise<() => void> {
    if (!hasTauriInternals()) {
      return () => undefined;
    }
    const unlisten = await listen<AmbiguityPromptPayload>(
      "qlevel://ambiguity-prompt",
      (e) => {
        handler(e.payload);
      }
    );
    return () => unlisten();
  }

  public async onPdfRasterRequest(
    handler: (ev: PdfRasterRequestEvent) => void
  ): Promise<() => void> {
    if (!hasTauriInternals()) {
      return () => undefined;
    }
    const unlisten = await listen<PdfRasterRequestEvent>(
      "qlevel://pdf-raster-request",
      (e) => {
        handler(e.payload);
      }
    );
    return () => unlisten();
  }

  public async onNativeFileDrop(
    handler: (ev: NativeFileDropPayload) => void
  ): Promise<() => void> {
    if (!hasTauriInternals()) {
      return () => undefined;
    }
    const webview = getCurrentWebview();
    const unlisten = await webview.onDragDropEvent((ev) => {
      const p = ev.payload;
      if (p.type === "enter" || p.type === "over") {
        const paths = "paths" in p ? p.paths : [];
        handler({ kind: "hover", paths });
      } else if (p.type === "drop") {
        handler({ kind: "drop", paths: p.paths });
      } else {
        handler({ kind: "cancel", paths: [] });
      }
    });
    return () => unlisten();
  }
}

export const qlevelClient = new QlevelClient();
