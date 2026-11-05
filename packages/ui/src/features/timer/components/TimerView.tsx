import { useCompute } from 'feature-react/state';
import React from 'react';
import { Badge } from '@/components';
import { cn } from '@/lib';
import { type TTimerViewCx } from '../TimerViewCx';
import { CountdownTimerActions, CountdownTimerDial } from './countdown';
import { PomodoroTimerActions, PomodoroTimerDial } from './pomodoro';
import { ProgressiveTimerActions, ProgressiveTimerDial } from './progressive';
import { TimeDisplay } from './TimeDisplay';

export const TimerView: React.FC<TTimerViewProps> = (props) => {
	const { cx, className, style, layout = 'fill' } = props;
	const speed = useCompute(cx.$config, (value) => value.dev.speed);
	const actionsClassName = layout === 'content' ? 'mt-6 shrink-0' : 'mt-auto';

	const renderDial = React.useCallback((): React.ReactNode => {
		switch (cx.timer.mode) {
			case 'countdown':
				return <CountdownTimerDial cx={cx} />;
			case 'pomodoro':
				return <PomodoroTimerDial cx={cx} />;
			case 'progressive':
				return <ProgressiveTimerDial cx={cx} />;
		}
	}, [cx]);

	const renderActions = React.useCallback((): React.ReactNode => {
		switch (cx.timer.mode) {
			case 'countdown':
				return <CountdownTimerActions cx={cx.timer} className={actionsClassName} />;
			case 'pomodoro':
				return <PomodoroTimerActions cx={cx.timer} className={actionsClassName} />;
			case 'progressive':
				return <ProgressiveTimerActions cx={cx.timer} className={actionsClassName} />;
		}
	}, [cx, actionsClassName]);

	return (
		<div
			className={cn(
				'flex flex-col items-center',
				layout === 'content' ? 'pb-6' : 'pb-8',
				className
			)}
			style={style}
		>
			{renderDial()}

			<TimeDisplay cx={cx} className="mt-4" />

			{speed > 1 && (
				<Badge variant="warning" className="mt-1 font-mono">
					{speed}x
				</Badge>
			)}

			{renderActions()}
		</div>
	);
};

interface TTimerViewProps {
	cx: TTimerViewCx;
	className?: string;
	style?: React.CSSProperties;
	layout?: 'fill' | 'content';
}
