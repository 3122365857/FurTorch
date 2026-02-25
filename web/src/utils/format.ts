/**
 * Formatting utilities
 */

export function formatTime(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  const pad = (n: number) => n.toString().padStart(2, "0")
  if (h > 0) {
    return `${h}:${pad(m)}:${pad(s)}`
  }
  return `${m}:${pad(s)}`
}

export function formatNumber(n: number): string {
  return new Intl.NumberFormat("ko-KR").format(Math.round(n))
}
