import '@testing-library/jest-dom';

// Mock Tauri APIs
const mockInvoke = vi.fn();
const mockEmit = vi.fn();
const mockListen = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

vi.mock('@tauri-apps/api/event', () => ({
  emit: mockEmit,
  listen: mockListen,
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(() => Promise.resolve('/mock/path')),
}));

// Global test utilities
global.mockInvoke = mockInvoke;
global.mockEmit = mockEmit;
global.mockListen = mockListen;

// Reset mocks before each test
beforeEach(() => {
  vi.clearAllMocks();
});
