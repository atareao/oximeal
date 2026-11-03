import { useEffect, useRef, useState } from 'react';
import { Alert, Button, Empty, Input, Modal, Skeleton } from 'antd';
import type { InputRef } from 'antd';
import type { MealPlanInput, MealRole, MealType, RecipeQueryParams } from '../api/types';
import { useRecipes } from '../api/hooks';
import {
  DEFAULT_SERVINGS,
  MEAL_ROLE_OPTIONS,
  MEAL_TYPE_LABELS,
  MEAL_TYPE_OPTIONS,
} from '../lib/mealPlan';
import './AddMealPlanDialog.css';

const { Search } = Input;

/** Retardo del debounce de búsqueda del recetario. */
const SEARCH_DEBOUNCE_MS = 300;
const PAGE_SIZE = 20;

export interface AddMealPlanDialogProps {
  open: boolean;
  date: string;
  /** Si se fija, el slot ya determina el tipo de comida (no se vuelve a pedir). */
  mealType?: MealType;
  online: boolean;
  submitting: boolean;
  /** Mensaje de error del servidor a mostrar dentro del diálogo. */
  error?: string | null;
  onClose: () => void;
  onSubmit: (input: MealPlanInput) => void;
}

/**
 * Selector de receta para añadir una entrada al plan. Reutiliza `useRecipes`
 * para buscar en el catálogo, permite elegir rol y raciones y valida en cliente
 * antes de delegar el envío en `onSubmit`.
 */
export function AddMealPlanDialog({
  open,
  date,
  mealType,
  online,
  submitting,
  error = null,
  onClose,
  onSubmit,
}: AddMealPlanDialogProps) {
  const [search, setSearch] = useState('');
  const [debouncedSearch, setDebouncedSearch] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [role, setRole] = useState<MealRole>('main');
  const [servings, setServings] = useState(String(DEFAULT_SERVINGS));
  const [localMealType, setLocalMealType] = useState<MealType>('lunch');
  const [validationError, setValidationError] = useState<string | null>(null);
  const [prevOpenKey, setPrevOpenKey] = useState<string | null>(null);

  const openerRef = useRef<HTMLElement | null>(null);
  const searchRef = useRef<InputRef | null>(null);

  // Gestión de foco: al abrir, el foco entra en el selector; al cerrar, vuelve
  // al control que lo abrió. Se hace con efectos (no solo `afterOpenChange`)
  // para funcionar también sin animaciones, como en los tests.
  useEffect(() => {
    if (!open) {
      openerRef.current?.focus();
      return undefined;
    }
    openerRef.current = document.activeElement as HTMLElement | null;
    const timer = window.setTimeout(() => searchRef.current?.focus(), 0);
    return () => window.clearTimeout(timer);
  }, [open]);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(search), SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [search]);

  const term = debouncedSearch.trim();
  const params: RecipeQueryParams = term
    ? { q: term, page: 1, page_size: PAGE_SIZE }
    : { page: 1, page_size: PAGE_SIZE };
  const { data, isLoading, isError, refetch } = useRecipes(params);

  // Reinicia el formulario al abrir (o al cambiar de slot) durante el render,
  // comparando con el valor previo, en lugar de con un efecto.
  const openKey = open ? `${date}|${mealType ?? ''}` : null;
  if (openKey !== prevOpenKey) {
    setPrevOpenKey(openKey);
    if (openKey !== null) {
      setSearch('');
      setDebouncedSearch('');
      setSelectedId(null);
      setRole('main');
      setServings(String(DEFAULT_SERVINGS));
      setLocalMealType(mealType ?? 'lunch');
      setValidationError(null);
    }
  }

  const items = data?.items ?? [];
  const effectiveMealType = mealType ?? localMealType;

  const title = mealType
    ? `Añadir receta a ${MEAL_TYPE_LABELS[mealType]}`
    : 'Planificar una comida';

  const handleConfirm = () => {
    if (!selectedId) {
      setValidationError('Selecciona una receta');
      return;
    }
    const parsedServings = Number(servings);
    if (!Number.isFinite(parsedServings) || parsedServings < 1) {
      setValidationError('Las raciones deben ser al menos 1');
      return;
    }
    setValidationError(null);
    onSubmit({
      date,
      meal_type: effectiveMealType,
      recipe_id: selectedId,
      role,
      servings: parsedServings,
    });
  };

  return (
    <Modal
      open={open}
      title={title}
      onCancel={onClose}
      footer={
        <div className="add-meal-plan__actions">
          <Button className="add-meal-plan__cancel" onClick={onClose}>
            Cancelar
          </Button>
          <Button
            type="primary"
            className="add-meal-plan__confirm"
            loading={submitting}
            disabled={!online}
            onClick={handleConfirm}
          >
            Añadir
          </Button>
        </div>
      }
    >
      {!online && (
        <Alert
          type="warning"
          showIcon
          data-testid="add-meal-plan-offline"
          className="add-meal-plan__offline"
          message="Sin conexión: la planificación requiere conexión."
        />
      )}

      {error && (
        <Alert
          type="error"
          showIcon
          data-testid="add-meal-plan-error"
          className="add-meal-plan__error"
          message={error}
        />
      )}

      {!mealType && (
        <div className="add-meal-plan__field">
          <label htmlFor="add-meal-plan-type" className="add-meal-plan__label">
            Tipo de comida
          </label>
          <select
            id="add-meal-plan-type"
            className="add-meal-plan__select"
            value={localMealType}
            onChange={(event) => setLocalMealType(event.target.value as MealType)}
          >
            {MEAL_TYPE_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>
      )}

      <Search
        ref={searchRef}
        aria-label="Buscar recetas"
        placeholder="Buscar recetas…"
        value={search}
        allowClear
        onChange={(event) => setSearch(event.target.value)}
        className="add-meal-plan__search"
      />

      {isLoading && (
        <div className="add-meal-plan__loading" data-testid="add-meal-plan-loading">
          <Skeleton active paragraph={{ rows: 3 }} />
        </div>
      )}

      {!isLoading && isError && (
        <div className="add-meal-plan__catalog-error">
          <p role="alert">No se pudieron cargar las recetas</p>
          <Button onClick={() => void refetch()}>Reintentar</Button>
        </div>
      )}

      {!isLoading && !isError && items.length === 0 && (
        <Empty description="Sin resultados" image={Empty.PRESENTED_IMAGE_SIMPLE} />
      )}

      {!isLoading && !isError && items.length > 0 && (
        <ul className="add-meal-plan__recipes" aria-label="Resultados de recetas">
          {items.map((recipe) => (
            <li key={recipe.id}>
              <button
                type="button"
                className={`add-meal-plan__recipe${
                  selectedId === recipe.id ? ' add-meal-plan__recipe--selected' : ''
                }`}
                aria-pressed={selectedId === recipe.id}
                onClick={() => setSelectedId(recipe.id)}
              >
                {recipe.name}
              </button>
            </li>
          ))}
        </ul>
      )}

      <div className="add-meal-plan__row">
        <div className="add-meal-plan__field">
          <label htmlFor="add-meal-plan-role" className="add-meal-plan__label">
            Rol
          </label>
          <select
            id="add-meal-plan-role"
            className="add-meal-plan__select"
            value={role}
            onChange={(event) => setRole(event.target.value as MealRole)}
          >
            {MEAL_ROLE_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>
        <div className="add-meal-plan__field">
          <label htmlFor="add-meal-plan-servings" className="add-meal-plan__label">
            Raciones
          </label>
          <Input
            id="add-meal-plan-servings"
            type="number"
            min={1}
            inputMode="numeric"
            value={servings}
            onChange={(event) => setServings(event.target.value)}
          />
        </div>
      </div>

      {validationError && (
        <p role="alert" className="add-meal-plan__validation">
          {validationError}
        </p>
      )}
    </Modal>
  );
}
