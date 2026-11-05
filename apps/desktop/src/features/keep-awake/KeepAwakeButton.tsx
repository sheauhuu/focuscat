import { cn, IconButton, MonitorIcon } from '@repo/ui';
import React from 'react';
import { specta } from '@/environment';
import { usePlatform } from '@/hooks';

export const KeepAwakeButton: React.FC<TKeepAwakeButtonProps> = (props) => {
	const { variant = 'bare' } = props;
	const platform = usePlatform();
	const [enabled, setEnabled] = React.useState<boolean | null>(null);
	const [pending, setPending] = React.useState(false);
	const [error, setError] = React.useState<string | null>(null);

	const handleToggle = React.useCallback(async () => {
		if (enabled == null || pending) return;
		setPending(true);
		setError(null);
		try {
			const result = await specta.commands.setKeepAwake(!enabled);
			if (result.status === 'error') throw new Error(result.error);
		} catch (cause) {
			console.error('Failed to change screen awake setting:', cause);
			setError("Couldn't change screen awake setting. Try again.");
		} finally {
			setPending(false);
		}
	}, [enabled, pending]);

	React.useEffect(() => {
		if (platform !== 'macos') return;

		let disposed = false;
		let receivedUpdate = false;
		let unlisten: (() => void) | undefined;
		const initialize = async () => {
			try {
				const stopListening = await specta.events.keepAwakeChangedEvent.listen((event) => {
					receivedUpdate = true;
					if (!disposed) setEnabled(event.payload);
				});
				if (disposed) {
					stopListening();
					return;
				}
				unlisten = stopListening;
				const current = await specta.commands.getKeepAwake();
				// An event received during the initial query is newer than its snapshot
				if (!disposed && !receivedUpdate) setEnabled(current);
			} catch (cause) {
				console.error('Failed to load screen awake setting:', cause);
				if (!disposed) setError("Couldn't load screen awake setting.");
			}
		};
		void initialize();
		return () => {
			disposed = true;
			unlisten?.();
		};
	}, [platform]);

	if (platform !== 'macos') return null;

	const title = enabled ? 'Screen awake is on · Click to turn off' : 'Keep screen awake';

	return (
		<>
			<IconButton
				variant={variant}
				size="sm"
				className={cn(
					enabled &&
						'bg-primary/10 text-primary hover:bg-primary/15 hover:text-primary active:text-primary'
				)}
				onClick={handleToggle}
				disabled={enabled == null || pending}
				aria-label="Keep screen awake"
				aria-pressed={enabled === true}
				title={error ?? title}
			>
				<MonitorIcon className="size-4" />
			</IconButton>
			<span className="sr-only" role="status">
				{error}
			</span>
		</>
	);
};

interface TKeepAwakeButtonProps {
	variant?: 'bare' | 'ghost';
}
