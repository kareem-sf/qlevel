import type { RenderedPageImageDto } from "@qlevel/bindings";
import { GlobalWorkerOptions, getDocument } from "pdfjs-dist";

// Configure pdf.js worker via bundled URL so strict Tauri CSP (`worker-src 'self' blob:`) succeeds offline.
GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url
).toString();

/**
 * Renders specific 1-based PDF pages from raw PDF bytes onto an offscreen `<canvas>`
 * at `targetDpi` (default 200 DPI) and encodes PNG base64 payloads for Multimodal vision OCR.
 */
export async function rasterizePdfPagesInMemory(
  pdfBytes: Uint8Array,
  pageNumbers: readonly number[],
  targetDpi = 200
): Promise<RenderedPageImageDto[]> {
  const loadingTask = getDocument({ data: pdfBytes });
  const pdfDoc = await loadingTask.promise;
  const scale = Math.max(1.0, targetDpi / 72.0);
  const results: RenderedPageImageDto[] = [];

  for (const pageNum of pageNumbers) {
    if (pageNum < 1 || pageNum > pdfDoc.numPages) {
      continue;
    }
    const page = await pdfDoc.getPage(pageNum);
    const viewport = page.getViewport({ scale });
    const canvas = document.createElement("canvas");
    const widthPx = Math.ceil(viewport.width);
    const heightPx = Math.ceil(viewport.height);
    canvas.width = widthPx;
    canvas.height = heightPx;

    const ctx = canvas.getContext("2d");
    if (!ctx) {
      continue;
    }

    await page.render({
      canvasContext: ctx,
      viewport,
    }).promise;

    const dataUrl = canvas.toDataURL("image/png");
    const commaIdx = dataUrl.indexOf(",");
    const base64Data = commaIdx >= 0 ? dataUrl.slice(commaIdx + 1) : dataUrl;

    results.push({
      pageNumber: pageNum,
      widthPx,
      heightPx,
      mimeType: "image/png",
      base64Data,
    });
  }

  await pdfDoc.destroy();
  return results;
}
