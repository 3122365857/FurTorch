/**
 * Stats panel: time, map count, income
 */

import { formatTime, formatNumber } from "../../../utils/format"

interface StatsPanelProps {
  totalTimeSec: number
  mapCount: number
  income: number
  incomeAll: number
}

export function StatsPanel({
  totalTimeSec,
  mapCount,
  income,
  incomeAll,
}: StatsPanelProps) {
  return (
    <div className="stats-panel">
      <div className="stat-card">
        <span className="stat-label">플레이 시간</span>
        <span className="stat-value">{formatTime(totalTimeSec)}</span>
      </div>
      <div className="stat-card">
        <span className="stat-label">맵 수</span>
        <span className="stat-value">{formatNumber(mapCount)}</span>
      </div>
      <div className="stat-card stat-income">
        <span className="stat-label">현재 순수익</span>
        <span className="stat-value">{formatNumber(income)} TTD</span>
      </div>
      <div className="stat-card stat-income">
        <span className="stat-label">누적 순수익</span>
        <span className="stat-value">{formatNumber(incomeAll)} TTD</span>
      </div>
    </div>
  )
}
