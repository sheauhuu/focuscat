import { useMemoCleanup, type TCatFace, type TCatFur, type TCatHat } from '@repo/ui';
import { createState } from 'feature-state';
import React from 'react';
import { specta } from '@/environment';

export class SettingsCx {
	private _unlisten?: () => void;

	public readonly $appSettings = createState<TAppSettings>({
		version: '0.0.4',
		launchAtLogin: false,
		features: {
			goals: true,
			activity: true,
			focus: true,
			catWindow: true,
			developer: false
		},
		appearance: {
			theme: 'auto'
		},
		audio: {
			session: {
				enabled: true,
				volume: 0.6
			},
			sessionEnd: {
				enabled: true,
				volume: 0.6
			},
			effects: {
				enabled: true,
				volume: 0.6
			}
		},
		developer: {
			cat: false,
			timerSpeed: 1
		},
		timer: {
			timerMode: 'pomodoro',
			pomodoro: {
				workDurationMinutes: 25,
				shortBreakMinutes: 5,
				longBreakMinutes: 15,
				sessionsBeforeLongBreak: 4,
				autoAdvance: false,
				autoAdvanceCountdownSeconds: 5,
				showSessionSetup: false
			},
			progressive: {
				ratings: [
					{
						key: 'distracted',
						label: 'Distracted',
						description: 'Hard to focus, lots of interruptions',
						suggestions: [
							{ workMinutes: 3, breakMinutes: 5 },
							{ workMinutes: 5, breakMinutes: 5 },
							{ workMinutes: 10, breakMinutes: 5 }
						]
					},
					{
						key: 'okay',
						label: 'Okay',
						description: 'Some focus, manageable',
						suggestions: [
							{ workMinutes: 10, breakMinutes: 5 },
							{ workMinutes: 15, breakMinutes: 5 },
							{ workMinutes: 20, breakMinutes: 5 }
						]
					},
					{
						key: 'focused',
						label: 'Focused',
						description: 'Solid focus throughout',
						suggestions: [
							{ workMinutes: 20, breakMinutes: 2 },
							{ workMinutes: 25, breakMinutes: 2 },
							{ workMinutes: 30, breakMinutes: 2 }
						]
					},
					{
						key: 'flow',
						label: 'Flow',
						description: 'Deep focus — keep going',
						suggestions: [
							{ workMinutes: 30, breakMinutes: null },
							{ workMinutes: 45, breakMinutes: null },
							{ workMinutes: 60, breakMinutes: null }
						]
					}
				],
				autoAdvance: false,
				autoAdvanceCountdownSeconds: 5,
				showSessionSetup: false
			},
			countdown: {
				durationMinutes: 25,
				showSessionSetup: false
			}
		},
		goals: {
			dailyGoalMinutes: 120
		},
		activity: {
			trackApps: true,
			trackWindows: true,
			trackBrowser: true
		},
		focus: {
			blockThreshold: 'distracting'
		},
		cat: {
			equippedFur: 'white',
			equippedFace: 'cute',
			equippedHat: null
		}
	});

	constructor() {
		this.init();
	}

	private async init(): Promise<void> {
		this.$appSettings.set((await specta.commands.getSettings()) as TAppSettings);
		this._unlisten = await specta.events.appSettingsChangedEvent.listen((event) => {
			this.$appSettings.set(event.payload as TAppSettings);
		});
	}

	public unmount(): void {
		this._unlisten?.();
	}

	public async update(updates: Partial<TAppSettings>): Promise<void> {
		const current = this.$appSettings.get();
		const updated = { ...current, ...updates };
		this.$appSettings.set(updated);
		await specta.commands.setSettings(updated);
	}
}

export type TAppSettings = Omit<specta.AppSettings, 'cat'> & { cat: TCatSettings };

export interface TCatSettings {
	equippedFur: TCatFur;
	equippedFace: TCatFace;
	equippedHat: TCatHat | null;
}

const ReactSettingsCx = React.createContext<SettingsCx | null>(null);

export const SettingsCxProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
	const cx = useMemoCleanup(() => {
		const settingsCx = new SettingsCx();
		return [settingsCx, () => settingsCx.unmount()];
	}, []);

	return <ReactSettingsCx.Provider value={cx}>{children}</ReactSettingsCx.Provider>;
};

export function useSettingsCx(): SettingsCx {
	const cx = React.useContext(ReactSettingsCx);
	if (cx == null) {
		throw new Error('useSettingsCx must be used within a SettingsCxProvider');
	}
	return cx;
}
