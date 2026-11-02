import { Layout, Menu } from 'antd';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { useMediaQuery } from '../hooks/useMediaQuery';
import { VALET_URL, hasValet } from '../lib/config';
import { BottomNav } from './BottomNav';
import './AppLayout.css';

const { Sider, Content } = Layout;

const DESKTOP_QUERY = '(min-width: 768px)';

const NAV_ITEMS = [
  { key: '/', label: '📅 Menú' },
  { key: '/shopping', label: '🛒 Compra' },
];

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
        <main className="app-mobile__content">
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
      </Sider>
      <Layout>
        <Content className="app-content">
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
}
