import { TimerCxProvider as BaseTimerCxProvider } from '@repo/ui';
import { useCompute } from 'feature-react/state';
import React from 'react';
import { useSettingsCx } from '@/features/settings';
import { CountdownTimerCx, PomodoroTimerCx, ProgressivePomodoroTimerCx } from './modes';

export const TimerCxProvider: React.FC<TTimerCxProviderProps> = (props) => {
	const { children, windowKind = 'main' } = props;
	const settingsCx = useSettingsCx();
	const timerMode = useCompute(settingsCx.$appSettings, (value) => value.timer.timerMode);

	const cx = React.useMemo(() => {
		if (timerMode === 'countdown') {
			return new CountdownTimerCx(settingsCx, windowKind);
		} else if (timerMode === 'progressive') {
			return new ProgressivePomodoroTimerCx(settingsCx, windowKind);
		} else {
			return new PomodoroTimerCx(settingsCx, windowKind);
		}
	}, [timerMode, settingsCx, windowKind]);

	React.useEffect(() => cx.mount(), [cx]);

	return <BaseTimerCxProvider value={cx}>{children}</BaseTimerCxProvider>;
};

interface TTimerCxProviderProps {
	children: React.ReactNode;
	windowKind?: 'main' | 'cat' | 'settings';
}
