import { type TSessionStartInput } from '@repo/ui';
import { createState } from 'feature-state';
import { specta } from '@/environment';
import { type SettingsCx } from '@/features/settings';
import { appendFlowReturnSearch, isMenubarWindow, toTuple } from '@/lib';

export abstract class BaseTimerCx {
	protected readonly _settingsCx: SettingsCx;
	protected readonly _windowKind: TTimerWindowKind;
	protected _nextWorkSeconds = 300;

	public readonly $status = createState<'idle' | 'running' | 'paused'>('idle');
	public readonly $sessionType = createState('pomodoro:work');
	public readonly $remainingSeconds = createState(0);
	public readonly $totalSeconds = createState(0);
	public readonly $overtimeSeconds = createState(0);
	public readonly $sessionsCompleted = createState(0);
	public readonly $startedAt = createState<Date | null>(null);

	constructor(settingsCx: SettingsCx, windowKind: TTimerWindowKind) {
		this._settingsCx = settingsCx;
		this._windowKind = windowKind;
	}

	public mount(): () => void {
		let disposed = false;
		let receivedUpdate = false;
		let unlisten: (() => void) | undefined;
		const initialize = async () => {
			try {
				const stopListening = await specta.events.timerUpdatedEvent.listen((event) => {
					receivedUpdate = true;
					if (!disposed) this.applyTimerUpdate(event.payload);
				});
				if (disposed) {
					stopListening();
					return;
				}
				unlisten = stopListening;
				const timer = await specta.commands.getTimer();
				// Events received during initialization take precedence over the snapshot
				if (!disposed && !receivedUpdate) this.applyTimerUpdate(timer);
			} catch (error) {
				console.error('Failed to initialize timer:', error);
			}
		};
		void initialize();
		return () => {
			disposed = true;
			unlisten?.();
		};
	}

	protected applyTimerUpdate(timer: specta.TimerDto): void {
		this._nextWorkSeconds = timer.nextWorkSeconds;
		if (this.$status.get() !== timer.status) this.$status.set(timer.status);
		if (this.$sessionType.get() !== timer.sessionType) this.$sessionType.set(timer.sessionType);
		if (this.$remainingSeconds.get() !== timer.remainingSeconds)
			this.$remainingSeconds.set(timer.remainingSeconds);
		if (this.$totalSeconds.get() !== timer.totalSeconds) this.$totalSeconds.set(timer.totalSeconds);
		if (this.$overtimeSeconds.get() !== timer.overtimeSeconds)
			this.$overtimeSeconds.set(timer.overtimeSeconds);
		if (this.$sessionsCompleted.get() !== timer.sessionsCompleted)
			this.$sessionsCompleted.set(timer.sessionsCompleted);
	}

	public async pause(): Promise<void> {
		const [ok, , err] = toTuple(await specta.commands.pauseTimer());
		if (!ok) console.error('Failed to pause timer:', err);
	}

	public async resume(): Promise<void> {
		const [ok, , err] = toTuple(await specta.commands.resumeTimer());
		if (ok) {
			this.$startedAt.set(new Date());
		} else {
			console.error('Failed to resume timer:', err);
		}
	}

	public async reset(): Promise<void> {
		const [ok, , err] = toTuple(await specta.commands.resetTimer());
		if (ok) {
			this.$startedAt.set(null);
		} else {
			console.error('Failed to reset timer:', err);
		}
	}

	public async complete(): Promise<void> {
		const [ok, , err] = toTuple(await specta.commands.completeTimer());
		if (ok) {
			this.$startedAt.set(null);
		} else {
			console.error('Failed to complete timer:', err);
		}
	}

	public async setDuration(minutes: number): Promise<void> {
		const [ok, , err] = toTuple(await specta.commands.setTimerDuration(minutes));
		if (!ok) console.error('Failed to set timer duration:', err);
	}

	protected _toSessionStartInput(input?: TSessionStartInput): specta.SessionStartInput | null {
		if (input == null) {
			return null;
		}

		return {
			intention: input.intention ?? null,
			profileIds: input.profileIds ?? null,
			blockThreshold: input.blockThreshold ?? null
		};
	}

	protected async _showMainFlow(path: string): Promise<void> {
		if (isMenubarWindow()) {
			await specta.commands.showMenubarAtPath(path);
			return;
		}
		const nextPath = appendFlowReturnSearch(
			path,
			this._windowKind === 'cat' ? { kind: 'cat' } : undefined
		);
		await specta.commands.showMainWindowAtPath(nextPath);
		if (this._windowKind === 'cat') {
			await specta.commands.hideCatWindow();
		}
	}
}

export type TTimerWindowKind = 'main' | 'cat' | 'settings';
