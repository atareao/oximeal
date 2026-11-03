import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider, onlineManager } from '@tanstack/react-query';
import {
  createMealPlanEntry,
  createRecipe,
  createShoppingItem,
  deleteMealPlanEntry,
  deleteRecipe,
  importRecipe,
  patchMealPlanEntry,
  patchShoppingItem,
  updateRecipe,
} from './client';
import {
  queryKeys,
  useAddMealPlanEntry,
  useAddShoppingItem,
  useCreateRecipe,
  useDeleteRecipe,
  useImportRecipe,
  useRemoveMealPlanEntry,
  useToggleShoppingItem,
  useUpdateMealPlanEntry,
  useUpdateRecipe,
} from './hooks';
import type {
  MealPlanEntryView,
  MealPlanInput,
  MealPlanPatchInput,
  Recipe,
  RecipeInput,
  ShoppingItem,
  ShoppingListGroup,
} from './types';
import { isOnline } from '../offline/connectivity';
import {
  configureMutationQueue,
  enqueueCreate,
  getQueuedMutations,
} from '../offline/mutationQueue';
import { createMutationStore, type MutationStore } from '../offline/mutationStore';

vi.mock('./client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./client')>();
  return {
    ...actual,
    createShoppingItem: vi.fn(),
    patchShoppingItem: vi.fn(),
    createRecipe: vi.fn(),
    updateRecipe: vi.fn(),
    deleteRecipe: vi.fn(),
    importRecipe: vi.fn(),
    createMealPlanEntry: vi.fn(),
    deleteMealPlanEntry: vi.fn(),
    patchMealPlanEntry: vi.fn(),
  };
});

vi.mock('../offline/connectivity', () => ({
  isOnline: vi.fn(() => true),
  useOnline: vi.fn(() => true),
}));

const mockedCreate = vi.mocked(createShoppingItem);
const mockedPatch = vi.mocked(patchShoppingItem);
const mockedIsOnline = vi.mocked(isOnline);

const originalOnLine = Object.getOwnPropertyDescriptor(Navigator.prototype, 'onLine');

let queryClient: QueryClient;

function wrapper({ children }: { children: ReactNode }) {
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

function localItem(id: string): ShoppingItem {
  return {
    id,
    name: 'Leche',
    amount: 1,
    unit: 'ud',
    category: 'Otros',
    checked: false,
    updated_at: new Date().toISOString(),
    pending: true,
  };
}

describe('mutaciones offline-aware', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal('indexedDB', undefined);
    configureMutationQueue(createMutationStore());
    mockedIsOnline.mockReturnValue(true);
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          // Los ítems optimistas se escriben sobre una clave sin observadores;
          // con `gcTime: 0` react-query los recolectaría en el siguiente tick y
          // la aserción sería una carrera.
          gcTime: Infinity,
        },
        mutations: { retry: false },
      },
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    if (originalOnLine) {
      Object.defineProperty(navigator, 'onLine', originalOnLine);
    }
  });

  it('sin conexión encola el alta y la aplica de forma optimista', async () => {
    mockedIsOnline.mockReturnValue(false);
    const { result } = renderHook(() => useAddShoppingItem(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync({
        name: 'Leche',
        amount: 1,
        unit: 'ud',
        category: 'Refrigerados',
      });
    });

    const data = queryClient.getQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all);
    expect(data).toHaveLength(1);
    expect(data?.[0].category).toBe('Refrigerados');
    expect(data?.[0].items[0]).toMatchObject({ name: 'Leche', pending: true });
    expect(data?.[0].items[0].id.startsWith('local:')).toBe(true);

    expect(getQueuedMutations()).toHaveLength(1);
    expect(mockedCreate).not.toHaveBeenCalled();
  });

  it('sin conexión cae al camino offline si el alta falla por red', async () => {
    mockedCreate.mockRejectedValue({ status: 0 });
    const { result } = renderHook(() => useAddShoppingItem(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync({ name: 'Pan', amount: 1, unit: 'ud' });
    });

    expect(getQueuedMutations()).toHaveLength(1);
    const data = queryClient.getQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all);
    expect(data?.[0].items[0]).toMatchObject({ name: 'Pan', pending: true });
  });

  it('el toggle de un ítem local sin conexión no llama al servidor', async () => {
    mockedIsOnline.mockReturnValue(false);
    const { localId } = await enqueueCreate({ name: 'Leche', amount: 1, unit: 'ud' });
    queryClient.setQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all, [
      { category: 'Otros', items: [localItem(localId)] },
    ]);

    const { result } = renderHook(() => useToggleShoppingItem(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync({ id: localId, patch: { checked: true } });
    });

    expect(mockedPatch).not.toHaveBeenCalled();
    const queue = getQueuedMutations();
    expect(queue).toHaveLength(1);
    expect(queue[0]).toMatchObject({ kind: 'create', checked: true });
  });

  it('no revierte el cambio encolado si el encolado falla (deferred === true)', async () => {
    // La red cae entre `onMutate` y `onError`: el cambio se difirió (no iba al
    // servidor), así que un fallo al encolar no debe revertir la UI.
    mockedIsOnline.mockReturnValue(false);
    const failingStore: MutationStore = {
      readMutations: vi.fn(async () => []),
      addMutation: vi.fn(async () => 1),
      putMutation: vi.fn(async () => {
        throw new Error('boom');
      }),
      deleteMutation: vi.fn(async () => undefined),
      clearMutations: vi.fn(async () => undefined),
    };
    configureMutationQueue(failingStore);

    const { localId } = await enqueueCreate({ name: 'Leche', amount: 1, unit: 'ud' });
    queryClient.setQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all, [
      { category: 'Otros', items: [{ ...localItem(localId), checked: false }] },
    ]);

    const { result } = renderHook(() => useToggleShoppingItem(), { wrapper });

    await act(async () => {
      await expect(
        result.current.mutateAsync({ id: localId, patch: { checked: true } }),
      ).rejects.toThrow('boom');
    });

    const data = queryClient.getQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all);
    expect(data?.[0].items[0]).toMatchObject({ checked: true, pending: true });
  });
});

describe('mutaciones de recetas', () => {
  const mockedCreateRecipe = vi.mocked(createRecipe);
  const mockedUpdateRecipe = vi.mocked(updateRecipe);
  const mockedDeleteRecipe = vi.mocked(deleteRecipe);

  const input: RecipeInput = {
    name: 'Tortilla',
    category: 'main',
    ingredients: [{ name: 'Patata', amount: 500, unit: 'g' }],
  };

  const recipe: Recipe = {
    id: 'r1',
    name: 'Tortilla',
    category: 'main',
    diet_type: 'mediterranean',
    prep_time_minutes: 25,
    servings: 4,
    created_at: '2026-11-01T10:00:00Z',
    ingredients: [{ name: 'Patata', amount: 500, unit: 'g' }],
    instructions: null,
    source_url: null,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
  });

  it('useCreateRecipe crea e invalida el catálogo', async () => {
    mockedCreateRecipe.mockResolvedValue(recipe);
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useCreateRecipe(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedCreateRecipe).toHaveBeenCalledWith(input);
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.recipes.all }),
    );
  });

  it('useUpdateRecipe actualiza e invalida catálogo y detalle', async () => {
    mockedUpdateRecipe.mockResolvedValue({ ...recipe, name: 'Nueva' });
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useUpdateRecipe(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync({ id: 'r1', input });
    });

    expect(mockedUpdateRecipe).toHaveBeenCalledWith('r1', input);
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.recipes.all }),
    );
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.recipes.detail('r1') }),
    );
  });

  it('useDeleteRecipe borra e invalida el catálogo', async () => {
    mockedDeleteRecipe.mockResolvedValue(undefined);
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useDeleteRecipe(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync('r1');
    });

    expect(mockedDeleteRecipe).toHaveBeenCalledWith('r1');
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.recipes.all }),
    );
  });
});

describe('importación de recetas', () => {
  const mockedImportRecipe = vi.mocked(importRecipe);

  const recipe: Recipe = {
    id: 'r1',
    name: 'Tortilla',
    category: 'main',
    diet_type: 'mediterranean',
    prep_time_minutes: 25,
    servings: 4,
    created_at: '2026-11-01T10:00:00Z',
    ingredients: [{ name: 'Patata', amount: 500, unit: 'g' }],
    instructions: null,
    source_url: 'https://example.com/r',
  };

  beforeEach(() => {
    vi.clearAllMocks();
    onlineManager.setOnline(true);
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
  });

  afterEach(() => {
    onlineManager.setOnline(true);
  });

  it('useImportRecipe importa e invalida el catálogo', async () => {
    mockedImportRecipe.mockResolvedValue({ recipe, created: true });
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useImportRecipe(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync({ url: 'https://example.com/r' });
    });

    expect(mockedImportRecipe).toHaveBeenCalledWith({ url: 'https://example.com/r' });
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.recipes.all }),
    );
  });

  it('sin conexión la importación queda pausada y no llama al endpoint', async () => {
    // Importar requiere que el servidor alcance el origen: sin conexión la
    // mutación se pausa (networkMode 'online' por defecto) en lugar de
    // aparentar éxito. La UI ya deshabilita el botón en ese estado.
    onlineManager.setOnline(false);
    const { result } = renderHook(() => useImportRecipe(), { wrapper });

    act(() => {
      result.current.mutate({ url: 'https://example.com/r' });
    });

    await waitFor(() => expect(result.current.isPaused).toBe(true));
    expect(mockedImportRecipe).not.toHaveBeenCalled();
    expect(result.current.data).toBeUndefined();
  });
});

describe('mutaciones del plan de comidas', () => {
  const mockedCreateMealPlan = vi.mocked(createMealPlanEntry);
  const mockedDeleteMealPlan = vi.mocked(deleteMealPlanEntry);
  const mockedPatchMealPlan = vi.mocked(patchMealPlanEntry);

  const input: MealPlanInput = {
    date: '2026-11-02',
    meal_type: 'lunch',
    recipe_id: 'r1',
    role: 'main',
    servings: 2,
  };

  const entry: MealPlanEntryView = {
    id: 'e1',
    date: '2026-11-02',
    meal_type: 'lunch',
    role: 'main',
    servings: 2,
    recipe: {
      id: 'r1',
      name: 'Lentejas',
      category: 'main',
      diet_type: 'omnivora',
      prep_time_minutes: 30,
      servings: 2,
      created_at: '2026-10-01T10:00:00Z',
    },
  };

  beforeEach(() => {
    vi.clearAllMocks();
    onlineManager.setOnline(true);
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
  });

  it('useAddMealPlanEntry crea e invalida plan y compra', async () => {
    mockedCreateMealPlan.mockResolvedValue(entry);
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useAddMealPlanEntry(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedCreateMealPlan).toHaveBeenCalledWith(input);
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.mealPlan.all }),
    );
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.shoppingList.all }),
    );
  });

  it('useRemoveMealPlanEntry borra e invalida plan y compra', async () => {
    mockedDeleteMealPlan.mockResolvedValue(undefined);
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useRemoveMealPlanEntry(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync('e1');
    });

    expect(mockedDeleteMealPlan).toHaveBeenCalledWith('e1');
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.mealPlan.all }),
    );
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.shoppingList.all }),
    );
  });

  it('useUpdateMealPlanEntry actualiza raciones e invalida plan y compra', async () => {
    const patch: MealPlanPatchInput = { servings: 4 };
    mockedPatchMealPlan.mockResolvedValue({ ...entry, servings: 4 });
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useUpdateMealPlanEntry(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync({ id: 'e1', input: patch });
    });

    expect(mockedPatchMealPlan).toHaveBeenCalledWith('e1', patch);
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.mealPlan.all }),
    );
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.shoppingList.all }),
    );
  });

  it('sin conexión el alta del plan no llama al servidor ni se encola', async () => {
    // A diferencia de la lista de la compra, la planificación requiere conexión:
    // la mutación se pausa (networkMode 'online') y no se encola en IndexedDB.
    onlineManager.setOnline(false);
    const { result } = renderHook(() => useAddMealPlanEntry(), { wrapper });

    act(() => {
      result.current.mutate(input);
    });

    await waitFor(() => expect(result.current.isPaused).toBe(true));
    expect(mockedCreateMealPlan).not.toHaveBeenCalled();
  });
});
