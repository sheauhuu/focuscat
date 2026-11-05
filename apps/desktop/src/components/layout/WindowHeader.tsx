import { Badge, cn, RefreshCcwIcon } from '@repo/ui';
import React from 'react';
import { useAppInfo, usePlatform, useUpdateChecker } from '@/hooks';
import { isMenubarWindow } from '@/lib';

export const WindowHeader: React.FC<TWindowHeaderProps> = (props) => {
	const { title, showBadge = true, children, className } = props;
	const platform = usePlatform();
	const isMenubar = isMenubarWindow();
	const appInfo = useAppInfo();
	const { updateAvailable, updateInfo, installing, install } = useUpdateChecker();

	const handleUpdateClick = React.useCallback(() => {
		void install();
	}, [install]);

	return (
		<header
			data-tauri-drag-region={isMenubar ? undefined : true}
			className={cn(
				'border-base-200 bg-base-50 flex h-8 shrink-0 items-center border-b select-none',
				platform === 'macos' && !isMenubar ? 'pl-[70px]' : 'pl-2',
				className
			)}
		>
			{title != null && <span className="text-base-600 ml-2 text-sm font-semibold">{title}</span>}
			{showBadge &&
				(appInfo.stage === 'dev' ? (
					<Badge variant="warning" className="ml-2">
						DEV
					</Badge>
				) : (
					appInfo.version.startsWith('v0.') &&
					appInfo.distribution !== 'appStore' && (
						<Badge className="ml-2 bg-purple-400/10 text-purple-400">BETA</Badge>
					)
				))}
			{updateAvailable && (
				<Badge
					variant={
						installing
							? 'neutral'
							: updateInfo.urgency === 'warning'
								? 'warning'
								: updateInfo.urgency === 'urgent'
									? 'error'
									: 'neutral'
					}
					className={cn('ml-2 p-1', !installing && 'cursor-pointer hover:opacity-80')}
					onClick={installing ? undefined : handleUpdateClick}
				>
					<RefreshCcwIcon size={12} className={installing ? 'animate-spin' : undefined} />
				</Badge>
			)}
			<div data-tauri-drag-region={isMenubar ? undefined : true} className="flex-1" />
			{children != null && <div className="flex items-center gap-0.5 pr-1">{children}</div>}
		</header>
	);
};

export interface TWindowHeaderProps {
	title?: string;
	showBadge?: boolean;
	children?: React.ReactNode;
	className?: string;
}
