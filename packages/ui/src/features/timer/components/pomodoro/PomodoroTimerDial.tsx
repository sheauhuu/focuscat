import React from 'react';
import { TriangleRightIcon } from '@/components';
import { type TTimerViewCx } from '../../TimerViewCx';
import { SessionWheel } from '../SessionWheel';
import { TimeDial } from '../TimeDial';

export const PomodoroTimerDial: React.FC<TProps> = (props) => {
	const { cx } = props;

	// MARK: - UI

	return (
		<div className="relative flex w-full items-center">
			{/* Border lines */}
			<div className="bg-base-200 absolute inset-x-0 top-0 z-20 h-px" />
			<div className="bg-base-200 absolute inset-x-0 bottom-0 z-20 h-px" />

			<TimeDial cx={cx} />

			<div className="bg-base-200 h-16 w-px" />

			{/* Session wheel */}
			<div className="relative mr-2">
				<SessionWheel
					cx={cx}
					className="[mask-image:linear-gradient(to_bottom,transparent,black_1.5rem,black_calc(100%_-_1.5rem),transparent)]"
				/>

				{/* Center indicator */}
				<div className="pointer-events-none absolute inset-y-0 left-0 z-30 flex items-center">
					<TriangleRightIcon
						width={6}
						height={10}
						preserveAspectRatio="none"
						className="text-base-300"
					/>
				</div>
			</div>
		</div>
	);
};

interface TProps {
	cx: TTimerViewCx;
}
