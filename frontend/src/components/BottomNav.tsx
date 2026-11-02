import { useLocation, useNavigate } from 'react-router-dom';
import { useHaptics } from '../hooks/useHaptics';
import { VALET_URL, hasValet } from '../lib/config';
import './BottomNav.css';

interface NavTab {
  key: string;
  label: string;
  icon: string;
  path: string;
}

const NAV_TABS: readonly NavTab[] = [
  { key: 'menu', label: 'Menú', icon: '📅', path: '/' },
  { key: 'recipes', label: 'Recetas', icon: '🍳', path: '/recipes' },
  { key: 'shopping', label: 'Compra', icon: '🛒', path: '/shopping' },
];

/** Navegación inferior fija para móvil (menú, compra y asistente si hay Valet). */
export function BottomNav() {
  const navigate = useNavigate();
  const location = useLocation();
  const haptics = useHaptics();

  const handleNavigate = (path: string) => {
    haptics(10);
    navigate(path);
  };

  return (
    <nav className="bottom-nav" data-testid="bottom-nav" aria-label="Navegación principal">
      {NAV_TABS.map((tab) => {
        const isActive =
          location.pathname === tab.path || location.pathname.startsWith(`${tab.path}/`);
        return (
          <button
            key={tab.key}
            type="button"
            className={`bottom-nav__tab${isActive ? ' bottom-nav__tab--active' : ''}`}
            aria-current={isActive ? 'page' : undefined}
            onClick={() => handleNavigate(tab.path)}
          >
            <span className="bottom-nav__icon" aria-hidden="true">
              {tab.icon}
            </span>
            <span className="bottom-nav__label">{tab.label}</span>
          </button>
        );
      })}
      {hasValet && (
        <a
          className="bottom-nav__tab"
          href={VALET_URL}
          target="_blank"
          rel="noopener noreferrer"
          onClick={() => haptics(10)}
        >
          <span className="bottom-nav__icon" aria-hidden="true">
            🤖
          </span>
          <span className="bottom-nav__label">Asistente</span>
        </a>
      )}
    </nav>
  );
}
