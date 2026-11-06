import { Outlet } from 'react-router-dom';
import { AppHeader } from './AppHeader';
import { OfflineBanner } from './OfflineBanner';
import './AppLayout.css';

/**
 * Shell de la aplicación: barra superior única, contenido de la ruta y aviso
 * de modo degradado. Sin panel lateral ni navegación inferior.
 */
export function AppLayout() {
  return (
    <div className="app-shell">
      <AppHeader />
      <main className="app-content">
        <OfflineBanner />
        <Outlet />
      </main>
    </div>
  );
}
