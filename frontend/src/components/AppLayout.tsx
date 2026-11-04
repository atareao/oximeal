import { Layout, Menu, Button } from 'antd';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { useSession } from '../auth/session';
import { useMediaQuery } from '../hooks/useMediaQuery';
import { VALET_URL, hasValet } from '../lib/config';
import { BottomNav } from './BottomNav';
import { OfflineBanner } from './OfflineBanner';
import './AppLayout.css';

const { Sider, Content } = Layout;

const DESKTOP_QUERY = '(min-width: 768px)';

const NAV_ITEMS = [
  { key: '/', label: '📅 Menú' },
  { key: '/recipes', label: '🍳 Recetas' },
  { key: '/shopping', label: '🛒 Compra' },
];

/**
 * Identidad de la sesión con control de cierre de sesión.
 * Solo se muestra con sesión autenticada; sin roles ni administración.
 */
function SessionControl() {
  const { status, name, logout } = useSession();

  if (status !== 'authenticated') {
    return null;
  }

  // La redirección al login la asume exclusivamente `AuthGuard` en cuanto la
  // sesión pasa a `anonymous`; redirigir aquí también provocaría una doble
  // navegación. Si el logout falla, el estado ya queda anónimo (`finally` en
  // SessionProvider) y `AuthGuard` redirige igualmente.
  const handleLogout = async () => {
    try {
      await logout();
    } catch {
      // Silenciado: la sesión ya quedó anónima y AuthGuard redirige.
    }
  };

  return (
    <div className="session-control" data-testid="session-identity">
      <span className="session-control__name" title={name}>
        {name ?? 'Sesión'}
      </span>
      <Button
        size="small"
        data-testid="logout-button"
        onClick={() => void handleLogout()}
        aria-label="Cerrar sesión"
      >
        Salir
      </Button>
    </div>
  );
}

/**
 * Shell responsive: Sider lateral en desktop, contenido a pantalla completa
 * con navegación inferior fija en móvil.
 */
export function AppLayout() {
  const isDesktop = useMediaQuery(DESKTOP_QUERY);
  const navigate = useNavigate();
  const location = useLocation();

  // `/shopping/lo-que-sea` debe mantener seleccionado `/shopping`.
  const selectedKey = `/${location.pathname.split('/')[1] ?? ''}`;

  if (!isDesktop) {
    return (
      <div className="app-mobile">
        <header className="app-mobile__topbar">
          <span className="app-mobile__brand">🍽️ OxiMeal</span>
          <SessionControl />
        </header>
        <main className="app-mobile__content">
          <OfflineBanner />
          <Outlet />
        </main>
        <BottomNav />
      </div>
    );
  }

  return (
    <Layout className="app-desktop">
      <Sider data-testid="app-sider" width={240} theme="light" className="app-sider">
        <div className="app-sider__brand">🍽️ OxiMeal</div>
        <Menu
          mode="inline"
          selectedKeys={[selectedKey]}
          onClick={({ key }) => navigate(key)}
          items={NAV_ITEMS}
        />
        {hasValet && (
          <div className="app-sider__assistant">
            <h3 className="app-sider__assistant-title">Asistente Valet</h3>
            <p className="app-sider__assistant-text">
              Planifica y consulta tus menús conversando con Valet.
            </p>
            <a
              className="app-sider__assistant-link"
              href={VALET_URL}
              target="_blank"
              rel="noopener noreferrer"
            >
              Abrir Valet ↗
            </a>
          </div>
        )}
        <SessionControl />
      </Sider>
      <Layout>
        <Content className="app-content">
          <OfflineBanner />
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
}
