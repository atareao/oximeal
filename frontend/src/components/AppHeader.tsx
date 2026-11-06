import { Link, NavLink } from 'react-router-dom';
import { useSession } from '../auth/session';
import { useMediaQuery } from '../hooks/useMediaQuery';
import './AppHeader.css';

const DESKTOP_QUERY = '(min-width: 768px)';
const WIDE_QUERY = '(min-width: 480px)';

interface NavItem {
  to: string;
  label: string;
  icon: string;
  end?: boolean;
}

const PRIMARY_ITEMS: readonly NavItem[] = [
  { to: '/', label: 'Menú', icon: '📅', end: true },
  { to: '/recipes', label: 'Recetas', icon: '🍳' },
  { to: '/shopping', label: 'Compra', icon: '🛒' },
];

const SETTINGS_ITEM: NavItem = { to: '/settings', label: 'Configuración', icon: '⚙️' };

const TOUCH_STYLE = { minHeight: 48 } as const;

/**
 * Barra de navegación superior única. En móvil muestra solo iconos con nombre
 * accesible; en desktop añade el texto visible.
 */
export function AppHeader() {
  const isDesktop = useMediaQuery(DESKTOP_QUERY);
  const isWide = useMediaQuery(WIDE_QUERY);
  const { status, logout } = useSession();

  const handleLogout = () => {
    void logout().catch(() => undefined);
  };

  return (
    <header className="app-header" data-testid="app-header">
      <Link
        to="/"
        className="app-header__brand"
        aria-label="OxiMeal"
        title="OxiMeal"
        style={TOUCH_STYLE}
      >
        {isWide ? (
          'OxiMeal'
        ) : (
          <span className="app-header__brand-mark" aria-hidden="true">
            OM
          </span>
        )}
      </Link>

      <nav className="app-header__nav" aria-label="Navegación principal">
        {PRIMARY_ITEMS.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.end}
            className="app-header__link"
            aria-label={item.label}
            title={item.label}
            style={TOUCH_STYLE}
          >
            <span className="app-header__icon" aria-hidden="true">
              {item.icon}
            </span>
            {isDesktop && <span className="app-header__label">{item.label}</span>}
          </NavLink>
        ))}
      </nav>

      <div className="app-header__actions">
        <NavLink
          to={SETTINGS_ITEM.to}
          className="app-header__link"
          aria-label={SETTINGS_ITEM.label}
          title={SETTINGS_ITEM.label}
          style={TOUCH_STYLE}
        >
          <span className="app-header__icon" aria-hidden="true">
            {SETTINGS_ITEM.icon}
          </span>
          {isDesktop && <span className="app-header__label">{SETTINGS_ITEM.label}</span>}
        </NavLink>
        {status === 'authenticated' && (
          <button
            type="button"
            className="app-header__logout"
            aria-label="Salir"
            title="Salir"
            style={TOUCH_STYLE}
            onClick={handleLogout}
          >
            <span className="app-header__icon" aria-hidden="true">
              🚪
            </span>
            {isDesktop && <span className="app-header__label">Salir</span>}
          </button>
        )}
      </div>
    </header>
  );
}
