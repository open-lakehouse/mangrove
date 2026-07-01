// Normalize errors thrown by openapi-fetch / the Unity Catalog REST API into a
// human-readable string. UC error bodies look like
// `{ "error_code": "...", "message": "..." , "details": [...] }`, but network or
// client failures surface as plain `Error`s, so we defensively handle both.

interface UcErrorBody {
  message?: string;
  // UC servers are inconsistent: the OSS REST server returns camelCase
  // `errorCode`, while some surfaces use snake_case `error_code`.
  error_code?: string;
  errorCode?: string;
  detail?: string;
  details?: string;
}

export function parseUcError(
  error: unknown,
  fallback = "Request failed.",
): string {
  if (!error) return fallback;

  if (typeof error === "string") return error || fallback;

  if (typeof error === "object") {
    const body = error as UcErrorBody & { error?: UcErrorBody };
    // openapi-fetch puts the parsed JSON error body under `error` on the result,
    // but our query layer throws that body directly, so check both shapes.
    const candidate = body.error ?? body;
    const message = candidate.message ?? candidate.detail ?? candidate.details;
    const code = candidate.error_code ?? candidate.errorCode;
    if (message) {
      return code ? `${code}: ${message}` : message;
    }
    if (error instanceof Error && error.message) return error.message;
  }

  return fallback;
}
