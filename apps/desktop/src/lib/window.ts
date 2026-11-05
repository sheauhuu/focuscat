import { getCurrentWindow } from '@tauri-apps/api/window';

export function isMenubarWindow(): boolean {
	return getCurrentWindow().label === 'menubar';
}
