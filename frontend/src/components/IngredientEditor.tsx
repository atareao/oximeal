import { useEffect, useRef, useState } from 'react';
import { Alert, App as AntdApp, Button, Input } from 'antd';
import type { InputRef } from 'antd';
import type { IngredientInput } from '../api/types';
import { parseAmount } from '../lib/ingredients';
import { formatScaledAmount, scaleAmount } from '../lib/scaling';

export interface IngredientEditorProps {
  /** Ingredientes de la receta, tal cual los devuelve la API. */
  ingredients: IngredientInput[];
  /** Factor de escalado por raciones (1 si no aplica). */
  factor: number;
  /** `true` para mostrar las cantidades escaladas en la vista de solo lectura. */
  showScaled: boolean;
  /** `true` si el usuario puede editar (hay conexión). */
  editable: boolean;
  /** `true` mientras se persiste el `PUT` de la receta. */
  saving: boolean;
  /** `true` cuando la lista está en modo edición (estado controlado por la página). */
  editing: boolean;
  onStartEdit: () => void;
  onCancel: () => void;
  /** Persiste la lista completa de ingredientes ya validada. */
  onSave: (ingredients: IngredientInput[]) => void;
}

interface IngredientDraft {
  /** Id estable del borrador: sirve de `key` y de ancla de foco. */
  id: string;
  /**
   * Número de fila asignado al crear el borrador. No se renumera al quitar una
   * fila intermedia, de modo que las etiquetas accesibles no cambian.
   */
  ordinal: number;
  name: string;
  amount: string;
  unit: string;
  category?: string;
}

/** Contador de ids de borrador; solo necesita ser único dentro de la lista. */
let ingredientIdSeq = 0;
function nextIngredientId(): string {
  ingredientIdSeq += 1;
  return `ingredient-${ingredientIdSeq}`;
}

function toDrafts(ingredients: IngredientInput[]): IngredientDraft[] {
  return ingredients.map((ingredient, index) => ({
    id: nextIngredientId(),
    ordinal: index + 1,
    name: ingredient.name,
    amount: String(ingredient.amount),
    unit: ingredient.unit,
    category: ingredient.category,
  }));
}

function nextOrdinal(drafts: IngredientDraft[]): number {
  return drafts.reduce((max, draft) => Math.max(max, draft.ordinal), 0) + 1;
}

/**
 * Lista de ingredientes del detalle con edición en línea.
 *
 * En modo lectura muestra nombre y cantidad (escalada si procede). En modo
 * edición convierte cada fila en campos de nombre/cantidad/unidad y permite
 * añadir y quitar filas; al guardar valida y delega la persistencia en la
 * página (que reutiliza `PUT /api/recipes/{id}` con la receta completa).
 *
 * Sin conexión, la edición queda degradada: el botón que abre el modo edición
 * y las acciones de la lista se deshabilitan, con un aviso explícito.
 */
export function IngredientEditor({
  ingredients,
  factor,
  showScaled,
  editable,
  saving,
  editing,
  onStartEdit,
  onCancel,
  onSave,
}: IngredientEditorProps) {
  const { message } = AntdApp.useApp();
  const [drafts, setDrafts] = useState<IngredientDraft[]>(() => toDrafts(ingredients));
  const toggleRef = useRef<HTMLButtonElement>(null);
  const nameRefs = useRef(new Map<string, InputRef>());
  const pendingFocus = useRef<string | null>(null);
  const wasEditing = useRef(false);

  // Al entrar en edición, reinicia el borrador desde la receta y pide enfocar
  // el primer campo; al salir, devuelve el foco al botón que abrió la edición.
  useEffect(() => {
    if (editing && !wasEditing.current) {
      wasEditing.current = true;
      const fresh = toDrafts(ingredients);
      setDrafts(fresh);
      pendingFocus.current = fresh[0]?.id ?? null;
    } else if (!editing && wasEditing.current) {
      wasEditing.current = false;
      toggleRef.current?.focus();
    }
  }, [editing, ingredients]);

  // Resuelve el foco pendiente tras el render (primer campo al editar o fila
  // contigua al quitar). No lleva deps: solo actúa cuando hay un foco pedido y
  // el nodo ya existe, para no perderlo si el render todavía no lo montó.
  useEffect(() => {
    const id = pendingFocus.current;
    if (!id) {
      return;
    }
    const node = nameRefs.current.get(id);
    if (node) {
      pendingFocus.current = null;
      node.focus();
    }
  });

  const heading = (
    <h3 id="recipe-ingredients-title" className="recipe-detail__section-title">
      Ingredientes
    </h3>
  );

  if (!editing) {
    return (
      <section aria-labelledby="recipe-ingredients-title" className="recipe-detail__section">
        <div className="recipe-detail__section-header">
          {heading}
          <Button
            ref={toggleRef}
            htmlType="button"
            className="recipe-detail__ingredients-edit"
            disabled={!editable}
            onClick={onStartEdit}
          >
            Corregir ingredientes
          </Button>
        </div>
        <ul className="recipe-detail__ingredients">
          {ingredients.map((ingredient, index) => (
            <li key={`${ingredient.name}-${index}`} className="recipe-detail__ingredient">
              <span className="recipe-detail__ingredient-name">{ingredient.name}</span>
              <span className="recipe-detail__ingredient-amount">
                {`${formatScaledAmount(
                  showScaled ? scaleAmount(ingredient.amount, factor) : ingredient.amount,
                )} ${ingredient.unit}`}
              </span>
            </li>
          ))}
        </ul>
      </section>
    );
  }

  const updateDraft = (index: number, patch: Partial<IngredientDraft>) => {
    setDrafts((current) =>
      current.map((draft, draftIndex) => (draftIndex === index ? { ...draft, ...patch } : draft)),
    );
  };

  const removeDraft = (index: number) => {
    const next = drafts.filter((_draft, draftIndex) => draftIndex !== index);
    // Enfoca la fila que ocupa la posición liberada (o la anterior si era la última).
    pendingFocus.current = next[Math.min(index, next.length - 1)]?.id ?? null;
    setDrafts(next);
  };

  const addDraft = () => {
    setDrafts((current) => [
      ...current,
      { id: nextIngredientId(), ordinal: nextOrdinal(current), name: '', amount: '', unit: '' },
    ]);
  };

  const handleSave = () => {
    if (!editable) {
      message.error('Sin conexión: no puedes guardar los cambios');
      return;
    }
    if (drafts.length === 0) {
      message.error('Añade al menos un ingrediente');
      return;
    }
    const parsed: IngredientInput[] = [];
    for (const draft of drafts) {
      const name = draft.name.trim();
      if (name.length === 0) {
        message.error('El nombre del ingrediente no puede estar vacío');
        return;
      }
      const amount = parseAmount(draft.amount);
      if (amount === null) {
        message.error('Indica una cantidad válida');
        return;
      }
      if (amount < 0) {
        message.error('La cantidad no puede ser negativa');
        return;
      }
      parsed.push({
        name,
        amount,
        unit: draft.unit.trim(),
        ...(draft.category ? { category: draft.category } : {}),
      });
    }
    onSave(parsed);
  };

  return (
    <section aria-labelledby="recipe-ingredients-title" className="recipe-detail__section">
      <div className="recipe-detail__section-header">{heading}</div>
      {!editable && (
        <Alert
          type="warning"
          showIcon
          data-testid="ingredient-editor-offline"
          message="Sin conexión: no puedes guardar los cambios de ingredientes."
        />
      )}
      <ul className="recipe-detail__ingredients recipe-detail__ingredients--editing">
        {drafts.map((draft, index) => (
          <li key={draft.id} className="recipe-detail__ingredient recipe-detail__ingredient--editing">
            <Input
              ref={(node) => {
                if (node) {
                  nameRefs.current.set(draft.id, node);
                } else {
                  nameRefs.current.delete(draft.id);
                }
              }}
              className="recipe-detail__ingredient-input recipe-detail__ingredient-input--name"
              value={draft.name}
              onChange={(event) => updateDraft(index, { name: event.target.value })}
              aria-label={`Nombre del ingrediente ${draft.ordinal}`}
              placeholder="Ingrediente"
            />
            <Input
              className="recipe-detail__ingredient-input recipe-detail__ingredient-input--amount"
              type="number"
              min={0}
              step="any"
              inputMode="decimal"
              value={draft.amount}
              onChange={(event) => updateDraft(index, { amount: event.target.value })}
              aria-label={`Cantidad del ingrediente ${draft.ordinal}`}
            />
            <Input
              className="recipe-detail__ingredient-input recipe-detail__ingredient-input--unit"
              value={draft.unit}
              onChange={(event) => updateDraft(index, { unit: event.target.value })}
              aria-label={`Unidad del ingrediente ${draft.ordinal}`}
              placeholder="ud"
            />
            <Button
              htmlType="button"
              danger
              disabled={!editable}
              className="recipe-detail__ingredient-remove"
              aria-label={`Eliminar ingrediente ${draft.ordinal}`}
              onClick={() => removeDraft(index)}
            >
              Quitar
            </Button>
          </li>
        ))}
      </ul>
      <div className="recipe-detail__ingredients-actions">
        <Button htmlType="button" disabled={!editable} onClick={addDraft}>
          Añadir ingrediente
        </Button>
        <Button htmlType="button" disabled={!editable || saving} onClick={onCancel}>
          Cancelar
        </Button>
        <Button
          type="primary"
          htmlType="button"
          className="recipe-detail__ingredients-save"
          disabled={!editable}
          loading={saving}
          onClick={handleSave}
        >
          Guardar ingredientes
        </Button>
      </div>
    </section>
  );
}
