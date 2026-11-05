import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { SettingsCx } from '@/features/settings';
import { BaseTimerCx } from './BaseTimerCx';

const mocks = vi.hoisted(() => ({
	getTimer: vi.fn(),
	listen: vi.fn()
}));

vi.mock('@/environment', () => ({
	specta: {
		commands: { getTimer: mocks.getTimer },
		events: { timerUpdatedEvent: { listen: mocks.listen } }
	}
}));

class TestTimerCx extends BaseTimerCx {}

describe('BaseTimerCx class', () => {
	const settings = {} as SettingsCx;
	const timer = {
		status: 'running',
		sessionType: 'progressive:break',
		remainingSeconds: 120,
		totalSeconds: 300,
		overtimeSeconds: 0,
		sessionsCompleted: 1,
		nextWorkSeconds: 1800
	};

	beforeEach(() => {
		vi.resetAllMocks();
		mocks.getTimer.mockResolvedValue(timer);
		mocks.listen.mockResolvedValue(vi.fn());
	});

	it('should subscribe only while mounted and ignore updates after cleanup', async () => {
		const stopListening = vi.fn();
		mocks.listen.mockResolvedValue(stopListening);
		const cx = new TestTimerCx(settings, 'cat');
		expect(mocks.getTimer).not.toHaveBeenCalled();
		const cleanup = cx.mount();
		await vi.waitFor(() => expect(cx.$remainingSeconds.get()).toBe(120));
		expect(cx.$remainingSeconds.get()).toBe(120);

		cleanup();
		const listener = mocks.listen.mock.calls[0]?.[0];
		listener({ payload: { ...timer, remainingSeconds: 119 } });
		expect(stopListening).toHaveBeenCalledOnce();
		expect(cx.$remainingSeconds.get()).toBe(120);
	});

	it('should ignore initialization from a disposed mount when the same instance remounts', async () => {
		let resolveFirst: (value: () => void) => void = () => {};
		mocks.listen.mockReturnValueOnce(
			new Promise<() => void>((resolve) => {
				resolveFirst = resolve;
			})
		);
		const cx = new TestTimerCx(settings, 'main');
		cx.mount()();
		const cleanup = cx.mount();
		await vi.waitFor(() => expect(cx.$remainingSeconds.get()).toBe(120));

		const stopFirstListener = vi.fn();
		resolveFirst(stopFirstListener);
		await Promise.resolve();
		expect(cx.$remainingSeconds.get()).toBe(120);
		expect(stopFirstListener).toHaveBeenCalledOnce();
		expect(mocks.getTimer).toHaveBeenCalledOnce();
		cleanup();
	});

	it('should keep an event received while the initial snapshot is pending', async () => {
		let resolveSnapshot: (value: typeof timer) => void = () => {};
		mocks.getTimer.mockReturnValueOnce(
			new Promise<typeof timer>((resolve) => {
				resolveSnapshot = resolve;
			})
		);
		const cx = new TestTimerCx(settings, 'main');
		const cleanup = cx.mount();
		await vi.waitFor(() => expect(mocks.getTimer).toHaveBeenCalledOnce());
		const listener = mocks.listen.mock.calls[0]?.[0];
		listener({ payload: { ...timer, remainingSeconds: 119 } });
		resolveSnapshot(timer);
		await Promise.resolve();

		expect(cx.$remainingSeconds.get()).toBe(119);
		cleanup();
	});

	it('should keep listening if the initial snapshot fails', async () => {
		const failure = new Error('Timer query failed');
		mocks.getTimer.mockRejectedValueOnce(failure);
		const logError = vi.spyOn(console, 'error').mockImplementation(() => {});
		const cx = new TestTimerCx(settings, 'main');
		const cleanup = cx.mount();
		try {
			await vi.waitFor(() =>
				expect(logError).toHaveBeenCalledWith('Failed to initialize timer:', failure)
			);
			const listener = mocks.listen.mock.calls[0]?.[0];
			listener({ payload: timer });
			expect(cx.$remainingSeconds.get()).toBe(120);
		} finally {
			cleanup();
			logError.mockRestore();
		}
	});
});
