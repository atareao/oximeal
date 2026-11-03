import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import type { RecipeSummary } from '../api/types';
import recipeCardCss from './RecipeCard.css?raw';
import { RecipeCard } from './RecipeCard';

function summary(overrides: Partial<RecipeSummary> = {}): RecipeSummary {
  return {
    id: 'r1',
    name: 'Tortilla',
    category: 'main',
    diet_type: 'mediterranean',
    prep_time_minutes: 20,
    servings: 2,
    created_at: '2026-11-01T10:00:00Z',
    tags: [],
    ...overrides,
  };
}

function renderCard(props: Partial<Parameters<typeof RecipeCard>[0]> = {}) {
  const onSelectTag = vi.fn();
  render(
    <MemoryRouter>
      <RecipeCard recipe={summary()} onSelectTag={onSelectTag} {...props} />
    </MemoryRouter>,
  );
  return { onSelectTag };
}

describe('RecipeCard', () => {
  it('muestra el nombre y enlaza al detalle', () => {
    renderCard({ recipe: summary({ id: 'r1', name: 'Tortilla' }) });

    expect(screen.getByRole('link', { name: /tortilla/i })).toHaveAttribute('href', '/recipes/r1');
  });

  it('muestra las etiquetas de la receta', () => {
    renderCard({
      recipe: summary({ tags: [{ id: 't1', name: 'Vegano', slug: 'vegano' }] }),
    });

    expect(screen.getByText('Vegano')).toBeInTheDocument();
    expect(screen.getByRole('list', { name: /etiquetas de tortilla/i })).toBeInTheDocument();
  });

  it('no muestra etiquetas si la receta no tiene', () => {
    renderCard({ recipe: summary({ tags: [] }) });

    expect(screen.queryByRole('list', { name: /etiquetas/i })).not.toBeInTheDocument();
  });

  it('al pulsar una etiqueta inicia el filtrado por ella', async () => {
    const user = userEvent.setup();
    const { onSelectTag } = renderCard({
      recipe: summary({ tags: [{ id: 't1', name: 'Vegano', slug: 'vegano' }] }),
    });

    await user.click(screen.getByRole('button', { name: /filtrar por etiqueta vegano/i }));

    expect(onSelectTag).toHaveBeenCalledWith('vegano');
  });

  it('la etiqueta es un objetivo táctil de 48px', () => {
    renderCard({
      recipe: summary({ tags: [{ id: 't1', name: 'Vegano', slug: 'vegano' }] }),
    });

    expect(screen.getByRole('button', { name: /filtrar por etiqueta vegano/i })).toHaveClass(
      'recipe-card__tag',
    );
    expect(recipeCardCss).toMatch(/\.recipe-card__tag[\s\S]*?min-height:\s*48px/);
  });
});
