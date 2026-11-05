import { isBreakSession, type TProgressivePomodoroCx, type TSessionStartInput } from '@repo/ui';
import { specta } from '@/environment';
import { toTuple } from '@/lib';
import { BaseTimerCx } from './BaseTimerCx';

export class ProgressivePomodoroTimerCx extends BaseTimerCx implements TProgressivePomodoroCx {
	public readonly mode = 'progressive' as const;

	public async start(input?: TSessionStartInput): Promise<void> {
		const s = this._settingsCx.$appSettings.get();
		if (s.timer.progressive.showSessionSetup && input == null) {
			await this._showMainFlow('/window/main/progressive/setup?advance=false');
			return;
		}
		const [ok, , err] = toTuple(await specta.commands.startTimer(this._toSessionStartInput(input)));
		if (ok) {
			this.$startedAt.set(new Date());
		} else {
			console.error('Failed to start progressive timer:', err);
		}
	}

	public async advance(input?: TSessionStartInput): Promise<void> {
		const sessionType = this.$sessionType.get();
		if (sessionType === 'progressive:work') {
			await this._showMainFlow('/window/main/progressive/rating');
			return;
		}

		const s = this._settingsCx.$appSettings.get();
		const isBreak = isBreakSession(this.$sessionType.get());
		if (s.timer.progressive.showSessionSetup && isBreak && input == null) {
			await this._showMainFlow('/window/main/progressive/setup?advance=true');
			return;
		}

		await this._advanceSession('Work', this._nextWorkSeconds, input);
	}

	public async advanceWithSuggestion(
		workSeconds: number,
		breakSeconds: number | null
	): Promise<void> {
		if (breakSeconds != null) {
			await this._advanceSession('Break', breakSeconds, undefined, workSeconds);
		} else {
			await this._advanceSession('Work', workSeconds, undefined, workSeconds);
		}
	}

	private async _advanceSession(
		sessionType: specta.ProgressiveSessionType,
		durationSeconds: number,
		input?: TSessionStartInput,
		nextWorkSeconds?: number
	): Promise<void> {
		const [ok, , err] = toTuple(
			await specta.commands.advanceProgressiveTimer(
				sessionType,
				durationSeconds,
				this._toSessionStartInput(input),
				nextWorkSeconds ?? null
			)
		);
		if (ok) {
			this.$startedAt.set(sessionType === 'Work' ? new Date() : null);
		} else {
			console.error('Failed to start next progressive session:', err);
		}
	}
}
