/**
 * Drop list: item name and quantity
 */

interface DropListProps {
  dropList: Record<string, number>
  maxRows?: number
}

export function DropList({ dropList, maxRows = 100 }: DropListProps) {
  const entries = Object.entries(dropList)
    .filter(([, qty]) => qty > 0)
    .sort((a, b) => b[1] - a[1])
    .slice(0, maxRows)

  if (entries.length === 0) {
    return (
      <div className="drop-list empty">
        <p>드롭된 아이템이 없습니다.</p>
        <p className="hint">게임 로그를 붙여넣어 처리하세요.</p>
      </div>
    )
  }

  return (
    <div className="drop-list">
      <div className="drop-list-header">
        <span>아이템</span>
        <span>수량</span>
      </div>
      <ul className="drop-list-items">
        {entries.map(([name, qty]) => (
          <li key={name} className="drop-item">
            <span className="drop-name">{name}</span>
            <span className="drop-qty">{qty}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}
