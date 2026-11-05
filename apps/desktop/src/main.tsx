import { RouterProvider } from '@tanstack/react-router';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { router } from './router';
import './styles.css';

// Expose router for Tauri navigation
// When Tauri needs to navigate an existing window, it calls:
//   window.__TAURI_ROUTER__.navigate({ to: '/path' })
// This enables SPA navigation without full page reload.
window.__TAURI_ROUTER__ = router;
if (window.__TAURI_PENDING_HREF__ != null) {
	const href = window.__TAURI_PENDING_HREF__;
	delete window.__TAURI_PENDING_HREF__;
	void router.navigate({ href });
}

declare global {
	interface Window {
		__TAURI_ROUTER__: typeof router;
		__TAURI_PENDING_HREF__?: string;
	}
}

createRoot(document.getElementById('root') as HTMLElement).render(
	<StrictMode>
		<RouterProvider router={router} />
	</StrictMode>
);
