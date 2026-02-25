/**
 * React hooks for TorchRust API
 */

import { useState, useEffect, useCallback } from "react"
import {
  getConfig,
  patchConfig,
  getStats,
  resetStats,
  processDrops,
  type Config,
  type Stats,
} from "./client"

const POLL_INTERVAL_MS = 2000

export function useConfig() {
  const [config, setConfig] = useState<Config | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await getConfig()
      setConfig(data)
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load config")
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    load()
  }, [load])

  const update = useCallback(
    async (updates: Partial<Config>) => {
      if (!config) return
      try {
        const updated = await patchConfig(updates)
        setConfig(updated)
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to update config")
      }
    },
    [config]
  )

  return { config, loading, error, reload: load, update }
}

export function useStats() {
  const [stats, setStats] = useState<Stats | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await getStats()
      setStats(data)
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load stats")
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    load()
    const id = setInterval(load, POLL_INTERVAL_MS)
    return () => clearInterval(id)
  }, [load])

  const reset = useCallback(async () => {
    try {
      await resetStats()
      await load()
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to reset stats")
    }
  }, [load])

  return { stats, loading, error, reload: load, reset }
}

export function useProcessDrops(onSuccess?: (stats: Stats) => void) {
  const [processing, setProcessing] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const process = useCallback(
    async (logText: string) => {
      if (!logText.trim()) return
      setProcessing(true)
      setError(null)
      try {
        const result = await processDrops(logText)
        onSuccess?.(result)
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to process drops")
      } finally {
        setProcessing(false)
      }
    },
    [onSuccess]
  )

  return { process, processing, error }
}
