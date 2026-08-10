const BYTES_PER_MEBIBYTE = 1024 * 1024;
const BYTES_PER_GIBIBYTE = BYTES_PER_MEBIBYTE * 1024;

export function formatMebibytes(bytes: number) {
  return `${(bytes / BYTES_PER_MEBIBYTE).toFixed(0)} MB`;
}

export function formatGibibytes(bytes: number) {
  return `${(bytes / BYTES_PER_GIBIBYTE).toFixed(1)} GB`;
}
