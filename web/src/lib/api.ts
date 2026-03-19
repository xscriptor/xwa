/**
 * Returns the base URL for the backend API.
 * Uses NEXT_PUBLIC_API_URL env var if set, otherwise defaults to localhost:8000.
 */
export function getApiBase(): string {
  return process.env.NEXT_PUBLIC_API_URL || "http://localhost:8000";
}

function buildApiUrl(path: string): string {
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  return `${getApiBase()}/api${normalizedPath}`;
}

async function parseJsonResponse<T>(resp: Response): Promise<T> {
  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(text || `Request failed with status ${resp.status}`);
  }
  return resp.json() as Promise<T>;
}

export interface TriggerScanResponse {
  message: string;
  scan_id: number;
}

export interface ReportSummary {
  id: number;
  target_url: string;
  urls_found: number;
  broken_links_count: number;
  missing_security_headers: number;
  is_ssl_valid: boolean;
  timestamp: string;
}

export interface InProgressReportResponse {
  status: "In Progress";
  current_step?: string;
}

export interface DeleteReportResponse {
  message: string;
  deleted_scan_id?: number;
  deleted_scan_ids?: number[];
  deleted_files: string[];
}

export type ReportResponse<T> = T | InProgressReportResponse;

export async function triggerScan(url: string): Promise<TriggerScanResponse> {
  const resp = await fetch(buildApiUrl("/scan"), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ url }),
  });
  return parseJsonResponse<TriggerScanResponse>(resp);
}

export async function listReports(skip = 0, limit = 20): Promise<ReportSummary[]> {
  const query = new URLSearchParams({ skip: String(skip), limit: String(limit) });
  const resp = await fetch(`${buildApiUrl("/reports")}?${query.toString()}`);
  return parseJsonResponse<ReportSummary[]>(resp);
}

export async function getReport<T>(scanId: string | number): Promise<ReportResponse<T>> {
  const resp = await fetch(buildApiUrl(`/reports/${scanId}`));
  return parseJsonResponse<ReportResponse<T>>(resp);
}

export async function deleteReport(scanId: string | number): Promise<DeleteReportResponse> {
  const resp = await fetch(buildApiUrl(`/reports/${scanId}`), {
    method: "DELETE",
  });
  return parseJsonResponse<DeleteReportResponse>(resp);
}

export async function deleteAllReports(): Promise<DeleteReportResponse> {
  const resp = await fetch(buildApiUrl("/reports"), {
    method: "DELETE",
  });
  return parseJsonResponse<DeleteReportResponse>(resp);
}

export function openProgressStream(scanId: string | number): EventSource {
  return new EventSource(buildApiUrl(`/progress/${scanId}`));
}

export function getMarkdownExportUrl(scanId: string | number): string {
  return buildApiUrl(`/export/md/${scanId}`);
}

export function getJsoncExportUrl(scanId: string | number): string {
  return buildApiUrl(`/export/jsonc/${scanId}`);
}
