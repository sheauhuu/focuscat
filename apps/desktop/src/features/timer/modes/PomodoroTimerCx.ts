import { isBreakSession, type TPomodoroCx, type TSessionStartInput } from '@repo/ui';
import { specta } from '@/environment';
import { toTuple } from '@/lib';
import { BaseTimerCx } from './BaseTimerCx';

export class PomodoroTimerCx extends BaseTimerCx implements TPomodoroCx {
	public readonly mode = 'pomodoro' as const;

	public async start(input?: TSessionStartInput): Promise<void> {
		const s = this._settingsCx.$appSettings.get();
		if (s.timer.pomodoro.showSessionSetup && input == null) {
			await this._showMainFlow('/window/main/pomodoro/setup?advance=false');
			return;
		}
		const [ok, , err] = toTuple(await specta.commands.startTimer(this._toSessionStartInput(input)));
		if (ok) {
			this.$startedAt.set(new Date());
		} else {
			console.error('Failed to start timer:', err);
		}
	}

	public async advance(input?: TSessionStartInput): Promise<void> {
		const s = this._settingsCx.$appSettings.get();
		const isBreak = isBreakSession(this.$sessionType.get());
		if (s.timer.pomodoro.showSessionSetup && isBreak && input == null) {
			await this._showMainFlow('/window/main/pomodoro/setup?advance=true');
			return;
		}
		const [ok, , err] = toTuple(
			await specta.commands.advancePomodoroTimer(this._toSessionStartInput(input))
		);
		if (ok) {
			this.$startedAt.set(null);
		} else {
			console.error('Failed to advance timer:', err);
		}
	}
}
