import { vi } from 'vitest';

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

// Mock @tauri-apps/api/event
vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn(() => Promise.resolve(() => {}))
}));

// Mock @tauri-apps/plugin-notification
vi.mock('@tauri-apps/plugin-notification', () => ({
	isPermissionGranted: vi.fn(() => Promise.resolve(true)),
	requestPermission: vi.fn(() => Promise.resolve('granted')),
	sendNotification: vi.fn()
}));
