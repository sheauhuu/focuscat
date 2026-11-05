import { createRootRoute, Outlet } from '@tanstack/react-router';
import React from 'react';
import { specta } from '@/environment';
import { isMenubarWindow } from '@/lib';

export const Route = createRootRoute({
	component: RootComponent
});

function RootComponent() {
	React.useEffect(() => {
		if (!isMenubarWindow()) return;
		const handleKeyDown = (event: KeyboardEvent) => {
			if (event.key === 'Escape' && !event.defaultPrevented) {
				void specta.commands.hideMenubar();
			}
		};
		window.addEventListener('keydown', handleKeyDown);
		return () => window.removeEventListener('keydown', handleKeyDown);
	}, []);

	return (
		<>
			<Outlet />
			{/* <TanStackRouterDevtools position="bottom-right" /> */}
		</>
	);
}
