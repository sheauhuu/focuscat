import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
	appendFlowReturnSearch,
	appendFlowReturnTargetParams,
	completeFlowReturn,
	completeFocusSettingsReturn,
	parseFlowReturnSearch,
	toFlowReturnTarget
} from './flow-return';

const mocks = vi.hoisted(() => ({
	isMenubar: vi.fn(),
	showMenubarAtPath: vi.fn(),
	showMainWindowAtPath: vi.fn(),
	showCatWindow: vi.fn(),
	hideMainWindow: vi.fn(),
	navigateMainWindowToPath: vi.fn(),
	hideSettingsWindow: vi.fn(),
	navigateSettingsWindowToPath: vi.fn()
}));

vi.mock('./window', () => ({ isMenubarWindow: mocks.isMenubar }));
vi.mock('@/environment', () => ({ specta: { commands: mocks } }));

describe('timer flow return functions', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		mocks.isMenubar.mockReturnValue(false);
	});

	it('should preserve the menu bar destination through profile editing search parameters', () => {
		const target = appendFlowReturnTargetParams(
			{ kind: 'menubar', href: '/window/main/pomodoro/setup?advance=true' },
			{ createdProfileId: '42' }
		);
		const href = appendFlowReturnSearch('/window/settings/focus/new', target);
		const search = Object.fromEntries(new URLSearchParams(href.split('?')[1]));

		expect(toFlowReturnTarget(parseFlowReturnSearch(search))).toEqual(target);
		expect(target.kind).toBe('menubar');
	});

	it('should return a menu bar flow to its panel without opening the main window or cat', async () => {
		mocks.isMenubar.mockReturnValue(true);
		await completeFlowReturn();

		expect(mocks.showMenubarAtPath).toHaveBeenCalledWith('/window/main');
		expect(mocks.showMainWindowAtPath).not.toHaveBeenCalled();
		expect(mocks.showCatWindow).not.toHaveBeenCalled();
	});

	it('should return from the Settings window to the specified menu bar page', async () => {
		await completeFocusSettingsReturn({ kind: 'menubar', href: '/window/main/progressive/setup' });

		expect(mocks.hideSettingsWindow).toHaveBeenCalledOnce();
		expect(mocks.showMenubarAtPath).toHaveBeenCalledWith('/window/main/progressive/setup');
		expect(mocks.showMainWindowAtPath).not.toHaveBeenCalled();
	});

	it('should keep an explicit main window destination even when called from the panel', async () => {
		mocks.isMenubar.mockReturnValue(true);
		await completeFlowReturn({ kind: 'main', href: '/window/main/countdown/setup' });

		expect(mocks.showMainWindowAtPath).toHaveBeenCalledWith('/window/main/countdown/setup');
		expect(mocks.showMenubarAtPath).not.toHaveBeenCalled();
	});

	it('should restore the cat and reset the hidden main page after a desktop cat flow', async () => {
		await completeFlowReturn({ kind: 'cat' });

		expect(mocks.showCatWindow).toHaveBeenCalledOnce();
		expect(mocks.hideMainWindow).toHaveBeenCalledOnce();
		expect(mocks.navigateMainWindowToPath).toHaveBeenCalledWith('/window/main');
		expect(mocks.showMenubarAtPath).not.toHaveBeenCalled();
	});
});
