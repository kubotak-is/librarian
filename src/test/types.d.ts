import type { MockedFunction } from 'vitest';

declare global {
  var mockInvoke: MockedFunction<(...args: unknown[]) => unknown>;
  var mockEmit: MockedFunction<(event: string, payload?: unknown) => void>;
  var mockListen: MockedFunction<(event: string, handler: (event: unknown) => void) => void>;
}
