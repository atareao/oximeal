import { useState } from 'react';
import { Button, Checkbox, Form, Input, Modal, Select } from 'antd';
import { useTags } from '../api/hooks';
import type { Recipe, RecipeInput, RecipeTag } from '../api/types';
import { RECIPE_CATEGORIES } from '../lib/recipes';
import './RecipeForm.css';

export interface RecipeFormProps {
  open: boolean;
  /** Receta a editar; `null`/ausente para el alta. */
  initial?: Recipe | null;
  submitting?: boolean;
  onClose: () => void;
  onSubmit: (input: RecipeInput) => void;
}

interface IngredientField {
  name?: string;
  amount?: string | number;
  unit?: string;
}

interface RecipeFormValues {
  name?: string;
  category?: string;
  diet_type?: string;
  prep_time_minutes?: string | number;
  servings?: string | number;
  instructions?: string;
  source_url?: string;
  ingredients?: IngredientField[];
  tags?: string[];
}

function toFormValues(recipe?: Recipe | null): RecipeFormValues {
  if (!recipe) {
    return {
      name: '',
      category: 'main',
      diet_type: 'mediterranean',
      servings: 2,
      ingredients: [{ name: '', amount: '', unit: '' }],
      tags: [],
    };
  }

  return {
    name: recipe.name,
    category: recipe.category,
    diet_type: recipe.diet_type,
    prep_time_minutes: recipe.prep_time_minutes ?? undefined,
    servings: recipe.servings,
    instructions: recipe.instructions ?? '',
    source_url: recipe.source_url ?? '',
    ingredients: recipe.ingredients.map((ingredient) => ({
      name: ingredient.name,
      amount: ingredient.amount,
      unit: ingredient.unit,
    })),
    // El payload envía nombres; las etiquetas se reemplazan al guardar.
    // Las automáticas se gestionan aparte (sugerencias retirables).
    tags: recipe.tags.filter((tag) => tag.source !== 'auto').map((tag) => tag.name),
  };
}

function toOptionalString(value?: string): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function toOptionalNumber(value?: string | number): number | undefined {
  if (value === undefined || value === null || value === '') {
    return undefined;
  }
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : undefined;
}

interface RecipeFormFieldsProps {
  initial: Recipe | null;
  submitting: boolean;
  onClose: () => void;
  onSubmit: (input: RecipeInput) => void;
}

/**
 * Cuerpo del formulario. `RecipeForm` lo monta con una `key` derivada de la
 * receta, de modo que su estado local se reinicie por remontaje (sin arrastrar
 * datos de la receta previa ni recurrir a `setState` dentro de un efecto).
 */
function RecipeFormFields({ initial, submitting, onClose, onSubmit }: RecipeFormFieldsProps) {
  const [form] = Form.useForm<RecipeFormValues>();
  const { data: tagsData } = useTags();
  const tagOptions = (tagsData ?? []).map((tag) => ({ value: tag.name, label: tag.name }));
  // Sugerencias automáticas de la receta en edición, aún no retiradas.
  const [suggestions, setSuggestions] = useState<RecipeTag[]>(() =>
    (initial?.tags ?? []).filter((tag) => tag.source === 'auto'),
  );
  // Slugs de sugerencias retiradas que se enviarán en `suppressed_auto_tags`.
  const [suppressed, setSuppressed] = useState<string[]>([]);
  const [autoTagsEnabled, setAutoTagsEnabled] = useState(true);

  const removeSuggestion = (tag: RecipeTag) => {
    setSuggestions((current) => current.filter((candidate) => candidate.id !== tag.id));
    setSuppressed((current) => (current.includes(tag.slug) ? current : [...current, tag.slug]));
  };

  const handleFinish = (values: RecipeFormValues) => {
    const input: RecipeInput = {
      name: values.name?.trim() ?? '',
      category: values.category ?? 'main',
      diet_type: toOptionalString(values.diet_type),
      prep_time_minutes: toOptionalNumber(values.prep_time_minutes),
      servings: toOptionalNumber(values.servings),
      ingredients: (values.ingredients ?? []).map((ingredient) => ({
        name: ingredient.name?.trim() ?? '',
        amount: toOptionalNumber(ingredient.amount) ?? 0,
        unit: ingredient.unit?.trim() ?? '',
      })),
      instructions: toOptionalString(values.instructions),
      source_url: toOptionalString(values.source_url),
      tags: (values.tags ?? []).map((tag) => tag.trim()).filter((tag) => tag.length > 0),
      auto_tags: autoTagsEnabled,
      suppressed_auto_tags: suppressed,
    };
    onSubmit(input);
  };

  return (
    <Form
      form={form}
      layout="vertical"
      initialValues={toFormValues(initial)}
      onFinish={handleFinish}
    >
      <Form.Item
        name="name"
        label="Nombre"
        rules={[{ required: true, message: 'Escribe el nombre de la receta' }]}
      >
        <Input autoComplete="off" placeholder="p. ej. Tortilla de patata" />
      </Form.Item>

      <Form.Item
        name="category"
        label="Categoría"
        rules={[{ required: true, message: 'Selecciona una categoría' }]}
      >
        <select className="recipe-form__select">
          {RECIPE_CATEGORIES.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </Form.Item>

      <Form.Item name="diet_type" label="Tipo de dieta">
        <Input placeholder="p. ej. mediterranean" />
      </Form.Item>

      <div className="recipe-form__row">
        <Form.Item name="prep_time_minutes" label="Tiempo de preparación (min)">
          <Input type="number" min={0} inputMode="numeric" />
        </Form.Item>
        <Form.Item name="servings" label="Raciones">
          <Input type="number" min={1} inputMode="numeric" />
        </Form.Item>
      </div>

      <Form.List
        name="ingredients"
        rules={[
          {
            validator: async (_rule, value: IngredientField[] | undefined) => {
              if (!value || value.length < 1) {
                throw new Error('Añade al menos un ingrediente');
              }
            },
          },
        ]}
      >
        {(fields, { add, remove }, { errors }) => (
          <div className="recipe-form__ingredients">
            <span className="recipe-form__ingredients-label">Ingredientes</span>
            {fields.map((field, index) => (
              <div key={field.key} className="recipe-form__ingredient">
                <Form.Item
                  name={[field.name, 'name']}
                  rules={[{ required: true, message: 'Indica el ingrediente' }]}
                  className="recipe-form__ingredient-field recipe-form__ingredient-field--name"
                >
                  <Input
                    aria-label={`Nombre del ingrediente ${index + 1}`}
                    placeholder="Ingrediente"
                  />
                </Form.Item>
                <Form.Item
                  name={[field.name, 'amount']}
                  rules={[{ required: true, message: 'Indica la cantidad' }]}
                  className="recipe-form__ingredient-field recipe-form__ingredient-field--amount"
                >
                  <Input
                    type="number"
                    min={0}
                    step="any"
                    inputMode="decimal"
                    aria-label={`Cantidad del ingrediente ${index + 1}`}
                  />
                </Form.Item>
                <Form.Item
                  name={[field.name, 'unit']}
                  rules={[{ required: true, message: 'Indica la unidad' }]}
                  className="recipe-form__ingredient-field recipe-form__ingredient-field--unit"
                >
                  <Input aria-label={`Unidad del ingrediente ${index + 1}`} placeholder="ud" />
                </Form.Item>
                <Button
                  htmlType="button"
                  onClick={() => remove(field.name)}
                  aria-label={`Eliminar ingrediente ${index + 1}`}
                  className="recipe-form__remove"
                >
                  Quitar
                </Button>
              </div>
            ))}
            <Button
              htmlType="button"
              onClick={() => add({ name: '', amount: '', unit: '' })}
              className="recipe-form__add"
            >
              Añadir ingrediente
            </Button>
            <Form.ErrorList errors={errors} />
          </div>
        )}
      </Form.List>

      <Form.Item name="instructions" label="Instrucciones">
        <Input.TextArea rows={4} placeholder="Cómo se elabora" />
      </Form.Item>

      <Form.Item name="source_url" label="URL de origen">
        <Input type="url" placeholder="https://…" />
      </Form.Item>

      <Form.Item name="tags" label="Etiquetas">
        <Select
          mode="tags"
          className="recipe-form__tags"
          aria-label="Etiquetas"
          placeholder="Añade etiquetas…"
          options={tagOptions}
          tokenSeparators={[',']}
        />
      </Form.Item>

      {suggestions.length > 0 && (
        <div className="recipe-form__auto-tags">
          <span className="recipe-form__auto-tags-label">Etiquetas sugeridas automáticamente</span>
          {/* eslint-disable-next-line jsx-a11y/no-redundant-roles -- el CSS list-style:none elimina el rol implícito en Safari/VoiceOver */}
          <ul
            role="list"
            className="recipe-form__suggestions"
            aria-label="Etiquetas sugeridas automáticamente"
          >
            {suggestions.map((tag) => (
              <li key={tag.id} className="recipe-form__suggestion">
                <span>{tag.name}</span>
                <Button
                  htmlType="button"
                  className="recipe-form__suggestion-remove"
                  aria-label={`Retirar ${tag.name}`}
                  onClick={() => removeSuggestion(tag)}
                >
                  Retirar
                </Button>
              </li>
            ))}
          </ul>
        </div>
      )}

      <Form.Item className="recipe-form__auto-tags-toggle">
        <Checkbox
          checked={autoTagsEnabled}
          onChange={(event) => setAutoTagsEnabled(event.target.checked)}
        >
          Etiquetado automático
        </Checkbox>
      </Form.Item>

      <div className="recipe-form__actions">
        <Button htmlType="button" onClick={onClose}>
          Cancelar
        </Button>
        <Button
          type="primary"
          htmlType="submit"
          loading={submitting}
          className="recipe-form__submit"
        >
          Guardar
        </Button>
      </div>
    </Form>
  );
}

/**
 * Formulario de alta/edición de receta, pensado para mostrarse en un `Modal`.
 * Valida nombre, categoría y al menos un ingrediente con nombre, cantidad y
 * unidad antes de delegar el envío en `onSubmit`.
 *
 * El cuerpo se monta con una `key` basada en el id de la receta y el modal se
 * destruye al ocultarse, de modo que al abrir/cambiar de receta el estado local
 * se reinicia por remontaje en lugar de mediante `setState` en un efecto.
 */
export function RecipeForm({
  open,
  initial = null,
  submitting = false,
  onClose,
  onSubmit,
}: RecipeFormProps) {
  return (
    <Modal
      open={open}
      title={initial ? 'Editar receta' : 'Nueva receta'}
      onCancel={onClose}
      footer={null}
      destroyOnHidden
    >
      <RecipeFormFields
        key={initial?.id ?? 'new'}
        initial={initial}
        submitting={submitting}
        onClose={onClose}
        onSubmit={onSubmit}
      />
    </Modal>
  );
}
