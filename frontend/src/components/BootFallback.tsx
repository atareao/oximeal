import './boot-fallback.css';

export interface BootFallbackProps {
  /** Texto mostrado junto al spinner mientras se carga el árbol de la app. */
  label?: string;
}

/**
 * Fallback de arranque **sin antd**: solo CSS y HTML, para que el primer
 * pintado no dependa del chunk de antd cargado de forma diferida.
 */
export function BootFallback({ label = 'Cargando OxiMeal…' }: BootFallbackProps) {
  return (
    <div className="boot-fallback" role="status" aria-live="polite" data-testid="boot-fallback">
      <span className="boot-fallback__spinner" aria-hidden="true" />
      <p className="boot-fallback__label">{label}</p>
    </div>
  );
}
