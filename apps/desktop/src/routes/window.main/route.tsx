import { createFileRoute, Outlet } from '@tanstack/react-router';
import { ThemeProvider } from '@/components';
import { SettingsCxProvider } from '@/features/settings';
import { TimerCxProvider } from '@/features/timer';

export const Route = createFileRoute('/window/main')({
	component: LayoutComponent
});

function LayoutComponent() {
	return (
		<SettingsCxProvider>
			<TimerCxProvider windowKind="main">
				<ThemeProvider>
					<Outlet />
				</ThemeProvider>
			</TimerCxProvider>
		</SettingsCxProvider>
	);
}
