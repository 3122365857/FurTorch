/**
 * TorchRust Dashboard - Main layout
 *
 * Displays: current/total time, map count, income, drop list, settings.
 */

import { useState } from "react"
import { useConfig, useStats, useProcessDrops } from "../../../api/hooks"
import { StatsPanel } from "./StatsPanel"
import { DropList } from "./DropList"
import { LogPaste } from "./LogPaste"
import { SettingsPanel } from "./SettingsPanel"

export function Dashboard() {
  const { config, loading: configLoading, error: configError, update } = useConfig()
  const { stats, loading: statsLoading, reload, reset } = useStats()
  const { process, processing, error: processError } = useProcessDrops(reload)
  const [settingsOpen, setSettingsOpen] = useState(false)

  const isLoading = configLoading || statsLoading
  const theme = config?.theme ?? {
    bg: "#1E1E1E",
    accent: "#3FFBFD",
    text: "#E6F1FF",
  }

  return (
    <div
      className="dashboard"
      style={
        {
          "--theme-bg": theme.bg,
          "--theme-accent": theme.accent,
          "--theme-text": theme.text,
        } as React.CSSProperties
      }
    >
      <header className="dashboard-header">
        <h1>TorchRust</h1>
        <p className="subtitle">Torchlight Infinite 드롭 트래커</p>
        <div className="header-actions">
          <button type="button" onClick={reset}>
            통계 초기화
          </button>
          <button type="button" onClick={() => setSettingsOpen(true)}>
            설정
          </button>
        </div>
      </header>

      {configError && (
        <div className="banner error">
          설정 로드 실패: {configError}
        </div>
      )}

      <main className="dashboard-main">
        {isLoading ? (
          <div className="loading">로딩 중...</div>
        ) : (
          <>
            <section className="stats-section">
              <StatsPanel
                totalTimeSec={stats?.total_time_sec ?? 0}
                mapCount={stats?.map_count ?? 0}
                income={stats?.income ?? 0}
                incomeAll={stats?.income_all ?? 0}
              />
            </section>

            <section className="content-grid">
              <div className="drop-section">
                <h2>드롭 목록</h2>
                <DropList dropList={stats?.drop_list ?? {}} />
              </div>

              <div className="log-section">
                <LogPaste
                  onProcess={process}
                  processing={processing}
                  error={processError}
                />
              </div>
            </section>
          </>
        )}
      </main>

      <SettingsPanel
        config={config}
        onUpdate={update}
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
      />
    </div>
  )
}
