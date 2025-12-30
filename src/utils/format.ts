/**
 * 秒数を "H:MM:SS" 形式にフォーマットする
 */
export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

/**
 * 秒数を "Xh Ym" 形式にフォーマットする（短い表示用）
 */
export function formatDurationShort(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

/**
 * ISO文字列を "HH:MM" 形式にフォーマットする
 */
export function formatTime(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleTimeString('ja-JP', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  });
}

/**
 * ISO文字列を "M/D (曜日)" 形式にフォーマットする
 */
export function formatDate(isoString: string): string {
  const date = new Date(isoString);
  const month = date.getMonth() + 1;
  const day = date.getDate();
  const weekdays = ['日', '月', '火', '水', '木', '金', '土'];
  const weekday = weekdays[date.getDay()];

  return `${month}/${day} (${weekday})`;
}

/**
 * ISO文字列を "YYYY-MM-DD" 形式にフォーマットする
 */
export function formatDateISO(isoString: string): string {
  const date = new Date(isoString);
  return date.toISOString().split('T')[0];
}

/**
 * 2つの日付間の秒数を計算する
 */
export function calculateDuration(startedAt: string, endedAt: string | null): number {
  const start = new Date(startedAt).getTime();
  const end = endedAt ? new Date(endedAt).getTime() : Date.now();
  return Math.floor((end - start) / 1000);
}

/**
 * 日付をグループ化キーに変換する（日単位）
 */
export function getDateGroupKey(isoString: string): string {
  return formatDateISO(isoString);
}

/**
 * 現在時刻をISO文字列で取得する
 */
export function nowISO(): string {
  return new Date().toISOString();
}

/**
 * 日付を datetime-local input用の形式に変換する
 */
export function toDatetimeLocal(isoString: string): string {
  const date = new Date(isoString);
  const offset = date.getTimezoneOffset();
  const localDate = new Date(date.getTime() - offset * 60 * 1000);
  return localDate.toISOString().slice(0, 16);
}

/**
 * datetime-local inputの値をISO文字列に変換する
 */
export function fromDatetimeLocal(datetimeLocal: string): string {
  return new Date(datetimeLocal).toISOString();
}
