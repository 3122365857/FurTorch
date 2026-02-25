/**
 * Settings panel for config
 */

import type { Config } from "../../../api/client"

interface SettingsPanelProps {
  config: Config | null
  onUpdate: (updates: Partial<Config>) => void
  open: boolean
  onClose: () => void
}

export function SettingsPanel({
  config,
  onUpdate,
  open,
  onClose,
}: SettingsPanelProps) {
  if (!config) return null

  return (
    <div className={`settings-overlay ${open ? "open" : ""}`} onClick={onClose}>
      <div className="settings-panel" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>설정</h2>
          <button type="button" className="close" onClick={onClose}>
            ×
          </button>
        </div>
        <div className="settings-body">
          <div className="setting-row">
            <label htmlFor="cost-per-map">맵당 비용 (TTD)</label>
            <input
              id="cost-per-map"
              type="number"
              min={0}
              step={1}
              value={config.cost_per_map ?? 0}
              onChange={(e) =>
                onUpdate({ cost_per_map: Number(e.target.value) || 0 })
              }
            />
          </div>
          <div className="setting-row">
            <label htmlFor="tax">거래세 적용</label>
            <select
              id="tax"
              value={config.tax === 1 ? "1" : "0"}
              onChange={(e) =>
                onUpdate({ tax: e.target.value === "1" ? 1 : 0 })
              }
            >
              <option value="1">예 (12.5%)</option>
              <option value="0">아니오</option>
            </select>
          </div>
          <div className="setting-row">
            <label htmlFor="realtime-hide-filter">실시간 숨김 필터 (쉼표 구분)</label>
            <input
              id="realtime-hide-filter"
              type="text"
              value={config.realtime_hide_filter ?? ""}
              onChange={(e) =>
                onUpdate({ realtime_hide_filter: e.target.value })
              }
              placeholder="예: 불꽃 가루, 진귀한 엠버"
            />
          </div>
        </div>
      </div>
    </div>
  )
}
