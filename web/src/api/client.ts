/**
 * TorchRust API client
 */

const API_BASE = "/api"

export interface Config {
  locale?: string
  cost_per_map?: number
  opacity?: number
  tax?: number
  realtime_hide_filter?: string
  theme?: {
    name: string
    bg: string
    accent: string
    text: string
  }
}

export interface Stats {
  drop_list: Record<string, number>
  drop_list_all: Record<string, number>
  income: number
  income_all: number
  map_count: number
  total_time_sec: number
}

export interface ProcessDropsResponse extends Stats {}

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const res = await fetch(url, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
  })
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error((err as { error?: string }).error ?? "Request failed")
  }
  return res.json()
}

export async function getConfig(): Promise<Config> {
  return fetchJson<Config>(`${API_BASE}/config`)
}

export async function patchConfig(updates: Partial<Config>): Promise<Config> {
  return fetchJson<Config>(`${API_BASE}/config`, {
    method: "PATCH",
    body: JSON.stringify(updates),
  })
}

export async function getStats(): Promise<Stats> {
  return fetchJson<Stats>(`${API_BASE}/stats`)
}

export async function resetStats(): Promise<void> {
  await fetchJson<{ ok: boolean }>(`${API_BASE}/stats/reset`, {
    method: "POST",
  })
}

export async function processDrops(logText: string): Promise<ProcessDropsResponse> {
  return fetchJson<ProcessDropsResponse>(`${API_BASE}/process-drops`, {
    method: "POST",
    body: JSON.stringify({ log_text: logText }),
  })
}
