export function timeify(ms: number): string {
  const h = ms / 3600000;
  const m = (ms % 3600000) / 60000;
  const s = (ms % 60000) / 1000;
  
  const hours = Math.floor(h);
  const minutes = Math.floor(m);
  const seconds = Math.floor(s);

  if (hours !== 0) {
    return `${hours}h${minutes !== 0 ? ` ${minutes}m` : ""}`;
  }

  if (minutes !== 0) {
    return `${minutes}m${seconds !== 0 ? ` ${seconds}s` : ""}`;
  }

  return `${+s.toFixed(2)}s`;
}
