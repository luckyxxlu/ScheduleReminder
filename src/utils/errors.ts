/**
 * Extracts a human-readable error message from an unknown error value.
 *
 * Tauri command errors are serialized as plain objects `{ message: "..." }`,
 * not as JavaScript `Error` instances. This helper handles both common shapes:
 *   - `Error` instance (browser-style thrown error)
 *   - plain object with a `message` string field (Tauri command rejection)
 * Falls back to `fallback` when no message can be extracted.
 */
export function extractErrorMessage(error: unknown, fallback: string): string {
  if (error instanceof Error) {
    return error.message
  }

  if (
    error !== null &&
    typeof error === 'object' &&
    'message' in error &&
    typeof (error as Record<string, unknown>).message === 'string' &&
    (error as Record<string, unknown>).message !== ''
  ) {
    return (error as Record<string, unknown>).message as string
  }

  return fallback
}
