import { invoke } from '@tauri-apps/api/core';
import type { ApiError } from './workspace-types';
export const ipc = <T>(command: string, request: object = {}): Promise<T> =>
  invoke<T>(command, { request });
export function apiError(error: unknown): ApiError {
  if (typeof error === 'object' && error !== null && 'message' in error) {
    const value = error as Partial<ApiError>;
    return {
      code: value.code || 'internal',
      message: String(value.message),
      details: value.details ?? null,
    };
  }
  return {
    code: 'internal',
    message: error instanceof Error ? error.message : String(error),
    details: null,
  };
}
export function errorHelp(error: ApiError): string {
  switch (error.code) {
    case 'stale_scope':
      return 'Preview the selection again before sending it.';
    case 'stale_evidence':
      return 'Read the changed source and run analysis again. Your edits are kept here.';
    case 'conflict':
      return 'Review the related changes in History. Resolve dependencies before retrying.';
    case 'runtime_unavailable':
      return 'Check the runtime status, then retry. You can still browse and create proposals.';
    case 'forbidden':
      return 'Local-only sources and topics cannot be sent to the runtime.';
    default:
      return '';
  }
}
