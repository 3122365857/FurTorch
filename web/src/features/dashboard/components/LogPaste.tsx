/**
 * Log paste area for processing drops
 */

import { useState } from "react"

interface LogPasteProps {
  onProcess: (text: string) => void
  processing: boolean
  error: string | null
}

export function LogPaste({ onProcess, processing, error }: LogPasteProps) {
  const [text, setText] = useState("")

  function handleSubmit() {
    onProcess(text)
    setText("")
  }

  return (
    <div className="log-paste">
      <label htmlFor="log-input">게임 로그 붙여넣기</label>
      <textarea
        id="log-input"
        value={text}
        onChange={(e) => setText(e.target.value)}
        placeholder="Torchlight Infinite 게임 로그를 여기에 붙여넣으세요..."
        rows={6}
        disabled={processing}
      />
      <div className="log-paste-actions">
        <button
          type="button"
          onClick={handleSubmit}
          disabled={!text.trim() || processing}
        >
          {processing ? "처리 중..." : "처리"}
        </button>
      </div>
      {error && <p className="error">{error}</p>}
    </div>
  )
}
