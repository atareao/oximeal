import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { App as AntdApp } from 'antd';
import type { ShoppingItem, ShoppingListGroup } from '../api/types';
import {
  clearCompletedShoppingItems,
  consolidateShoppingList,
  createShoppingItem,
  fetchShoppingList,
  patchShoppingItem,
} from '../api/client';
import { ShoppingPage } from './ShoppingPage';
import { configureMutationQueue, enqueueCreate } from '../offline/mutationQueue';
import { createMutationStore } from '../offline/mutationStore';

// Se mockea el cliente HTTP, no los hooks: así se ejerce la integración real
// (React Query + actualización optimista + rollback) de la página.
vi.mock('../api/client', () => ({
  fetchShoppingList: vi.fn(),
  patchShoppingItem: vi.fn(),
  createShoppingItem: vi.fn(),
  clearCompletedShoppingItems: vi.fn(),
  consolidateShoppingList: vi.fn(),
}));

const mockedFetchList = vi.mocked(fetchShoppingList);
const mockedPatch = vi.mocked(patchShoppingItem);
const mockedAdd = vi.mocked(createShoppingItem);
const mockedClear = vi.mocked(clearCompletedShoppingItems);
const mockedConsolidate = vi.mocked(consolidateShoppingList);

function item(overrides: Partial<ShoppingItem> & Pick<ShoppingItem, 'id' | 'name'>): ShoppingItem {
  return {
    amount: 1,
    unit: 'ud',
    category: 'Verdulería',
    checked: false,
    updated_at: '2026-11-01T10:00:00Z',
    ...overrides,
  };
}

/** Estado mutable que simula la "base de datos" del backend. */
let listState: ShoppingListGroup[] = [];

function applyPatch(id: string, patch: Partial<ShoppingItem>): ShoppingItem {
  for (const group of listState) {
    const found = group.items.find((entry) => entry.id === id);
    if (found) {
      Object.assign(found, patch);
      return { ...found };
    }
  }
  throw new Error(`ítem desconocido: ${id}`);
}

function renderShopping() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
      mutations: { retry: false },
    },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <AntdApp>
        <ShoppingPage />
      </AntdApp>
    </QueryClientProvider>,
  );
}

describe('ShoppingPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listState = [];
    configureMutationQueue(createMutationStore());
    vi.stubGlobal('navigator', { ...navigator, vibrate: vi.fn() });
    mockedFetchList.mockImplementation(() => Promise.resolve(structuredClone(listState)));
    mockedPatch.mockImplementation((id, patch) => Promise.resolve(applyPatch(id, patch)));
    mockedAdd.mockResolvedValue(item({ id: 'new', name: 'Nuevo' }));
    mockedClear.mockResolvedValue(undefined);
    mockedConsolidate.mockResolvedValue([]);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('agrupa los ítems por pasillo con su emoji', async () => {
    listState = [
      { category: 'Verdulería', items: [item({ id: '1', name: 'Tomate' })] },
      {
        category: 'Pescadería',
        items: [item({ id: '2', name: 'Salmón', category: 'Pescadería' })],
      },
    ];
    renderShopping();

    expect(await screen.findByRole('heading', { name: /Verdulería/ })).toBeInTheDocument();
    expect(screen.getByText('🥦')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: /Pescadería/ })).toBeInTheDocument();
    expect(screen.getByText('🐟')).toBeInTheDocument();
  });

  it('marca un ítem como comprado y lo mueve al final de su categoría', async () => {
    listState = [
      {
        category: 'Verdulería',
        items: [
          item({ id: '1', name: 'Tomate', updated_at: '2026-11-01T09:00:00Z' }),
          item({ id: '2', name: 'Lechuga', updated_at: '2026-11-01T10:00:00Z' }),
        ],
      },
    ];
    const user = userEvent.setup();
    renderShopping();

    await user.click(await screen.findByRole('checkbox', { name: /Tomate/ }));

    expect(mockedPatch).toHaveBeenCalledWith('1', { checked: true });

    await waitFor(() => {
      const rows = screen.getAllByRole('checkbox');
      expect(rows[0]).toHaveAccessibleName(/Lechuga/);
      expect(rows[1]).toHaveAccessibleName(/Tomate/);
    });
  });

  it('revierte el estado previo si el toggle falla (rollback)', async () => {
    listState = [
      {
        category: 'Verdulería',
        items: [
          item({ id: '1', name: 'Tomate', updated_at: '2026-11-01T09:00:00Z' }),
          item({ id: '2', name: 'Lechuga', updated_at: '2026-11-01T10:00:00Z' }),
        ],
      },
    ];
    mockedPatch.mockRejectedValue(new Error('boom'));
    const user = userEvent.setup();
    renderShopping();

    const tomate = await screen.findByRole('checkbox', { name: /Tomate/ });
    await user.click(tomate);

    await waitFor(() => {
      const rows = screen.getAllByRole('checkbox');
      expect(rows[0]).toHaveAccessibleName(/Tomate/);
      expect(rows[0]).not.toBeChecked();
    });
  });

  it('con dos toggles rápidos deja un estado final coherente', async () => {
    listState = [{ category: 'Verdulería', items: [item({ id: '1', name: 'Tomate' })] }];
    const user = userEvent.setup();
    renderShopping();

    const tomate = await screen.findByRole('checkbox', { name: /Tomate/ });
    await user.click(tomate);
    await user.click(tomate);

    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: /Tomate/ })).not.toBeChecked();
    });
    expect(mockedPatch).toHaveBeenCalledTimes(2);
    expect(mockedPatch).toHaveBeenNthCalledWith(1, '1', { checked: true });
    expect(mockedPatch).toHaveBeenNthCalledWith(2, '1', { checked: false });
  });

  it('da de alta un ítem con los datos del formulario', async () => {
    const user = userEvent.setup();
    renderShopping();

    await user.type(await screen.findByLabelText(/nombre del producto/i), 'Leche');
    const amount = screen.getByLabelText(/cantidad/i);
    await user.clear(amount);
    await user.type(amount, '2');
    await user.selectOptions(screen.getByLabelText(/unidad/i), 'l');
    await user.click(screen.getByRole('button', { name: /^añadir$/i }));

    await waitFor(() => {
      expect(mockedAdd).toHaveBeenCalledWith({ name: 'Leche', amount: 2, unit: 'l' });
    });
  });

  it('deshabilita "Limpiar comprados" cuando no hay comprados', async () => {
    listState = [{ category: 'Verdulería', items: [item({ id: '1', name: 'Tomate' })] }];
    renderShopping();

    await screen.findByText('Tomate');
    expect(screen.getByRole('button', { name: /limpiar comprados/i })).toBeDisabled();
  });

  it('habilita "Limpiar comprados" cuando hay comprados', async () => {
    listState = [
      { category: 'Verdulería', items: [item({ id: '1', name: 'Tomate', checked: true })] },
    ];
    renderShopping();

    await screen.findByText('Tomate');
    expect(screen.getByRole('button', { name: /limpiar comprados/i })).toBeEnabled();
  });

  it('"Limpiar comprados" pide confirmación antes de mutar', async () => {
    listState = [
      { category: 'Verdulería', items: [item({ id: '1', name: 'Tomate', checked: true })] },
    ];
    const user = userEvent.setup();
    renderShopping();

    await screen.findByText('Tomate');
    await user.click(screen.getByRole('button', { name: /limpiar comprados/i }));

    expect(mockedClear).not.toHaveBeenCalled();
    await user.click(await screen.findByRole('button', { name: 'Eliminar' }));

    await waitFor(() => {
      expect(mockedClear).toHaveBeenCalledTimes(1);
    });
  });

  it('"Marcar todo" marca los pendientes y no toca los ya marcados', async () => {
    listState = [
      {
        category: 'Verdulería',
        items: [
          item({ id: '1', name: 'Tomate' }),
          item({ id: '2', name: 'Lechuga', checked: true }),
          item({ id: '3', name: 'Zanahoria' }),
        ],
      },
    ];
    const user = userEvent.setup();
    renderShopping();

    await screen.findByText('Tomate');
    await user.click(screen.getByRole('button', { name: 'Marcar todo' }));

    await waitFor(() => {
      expect(mockedPatch).toHaveBeenCalledTimes(2);
    });
    expect(mockedPatch).toHaveBeenCalledWith('1', { checked: true });
    expect(mockedPatch).toHaveBeenCalledWith('3', { checked: true });
    expect(mockedPatch).not.toHaveBeenCalledWith('2', expect.anything());
    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: /Tomate/ })).toBeChecked();
      expect(screen.getByRole('checkbox', { name: /Zanahoria/ })).toBeChecked();
    });
  });

  it('deshabilita "Marcar todo" cuando no hay pendientes', async () => {
    listState = [
      { category: 'Verdulería', items: [item({ id: '1', name: 'Tomate', checked: true })] },
    ];
    renderShopping();

    await screen.findByText('Tomate');
    expect(screen.getByRole('button', { name: 'Marcar todo' })).toBeDisabled();
  });

  it('habilita "Marcar todo" cuando hay pendientes', async () => {
    listState = [{ category: 'Verdulería', items: [item({ id: '1', name: 'Tomate' })] }];
    renderShopping();

    await screen.findByText('Tomate');
    expect(screen.getByRole('button', { name: 'Marcar todo' })).toBeEnabled();
  });

  it('"Marcar todo" con 3 pendientes los marca todos en un lote secuencial', async () => {
    listState = [
      {
        category: 'Verdulería',
        items: [
          item({ id: '1', name: 'Tomate' }),
          item({ id: '2', name: 'Lechuga' }),
          item({ id: '3', name: 'Zanahoria' }),
        ],
      },
    ];
    const user = userEvent.setup();
    renderShopping();

    await screen.findByText('Tomate');
    await user.click(screen.getByRole('button', { name: 'Marcar todo' }));

    await waitFor(() => {
      expect(mockedPatch).toHaveBeenCalledTimes(3);
    });
    expect(mockedPatch).toHaveBeenCalledWith('1', { checked: true });
    expect(mockedPatch).toHaveBeenCalledWith('2', { checked: true });
    expect(mockedPatch).toHaveBeenCalledWith('3', { checked: true });
    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: /Tomate/ })).toBeChecked();
      expect(screen.getByRole('checkbox', { name: /Lechuga/ })).toBeChecked();
      expect(screen.getByRole('checkbox', { name: /Zanahoria/ })).toBeChecked();
    });
  });

  it('"Marcar todo" continúa el lote cuando un ítem falla (rollback parcial)', async () => {
    mockedPatch.mockImplementation((id, patch) =>
      id === '2' ? Promise.reject(new Error('boom')) : Promise.resolve(applyPatch(id, patch)),
    );
    listState = [
      {
        category: 'Verdulería',
        items: [
          item({ id: '1', name: 'Tomate' }),
          item({ id: '2', name: 'Lechuga' }),
          item({ id: '3', name: 'Zanahoria' }),
        ],
      },
    ];
    const user = userEvent.setup();
    renderShopping();

    await screen.findByText('Tomate');
    await user.click(screen.getByRole('button', { name: 'Marcar todo' }));

    await waitFor(() => {
      expect(mockedPatch).toHaveBeenCalledTimes(3);
    });
    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: /Tomate/ })).toBeChecked();
      expect(screen.getByRole('checkbox', { name: /Zanahoria/ })).toBeChecked();
      expect(screen.getByRole('checkbox', { name: /Lechuga/ })).not.toBeChecked();
    });
  });

  it('ignora toggles manuales mientras "Marcar todo" está en curso', async () => {
    const resolvers: Array<() => void> = [];
    mockedPatch.mockImplementation(
      (id, patch) =>
        new Promise<ShoppingItem>((resolve) => {
          resolvers.push(() => resolve(applyPatch(id, patch)));
        }),
    );
    listState = [
      {
        category: 'Verdulería',
        items: [item({ id: '1', name: 'Tomate' }), item({ id: '2', name: 'Lechuga' })],
      },
    ];
    const user = userEvent.setup();
    renderShopping();

    await screen.findByText('Tomate');
    await user.click(screen.getByRole('button', { name: 'Marcar todo' }));

    // La primera mutación del lote queda en vuelo (promesa sin resolver).
    await waitFor(() => {
      expect(mockedPatch).toHaveBeenCalledTimes(1);
    });

    // Un toggle manual durante el lote no dispara otra petición (guard).
    await user.click(screen.getByRole('checkbox', { name: /Lechuga/ }));
    expect(mockedPatch).toHaveBeenCalledTimes(1);

    // Drenamos el lote resolviendo las peticiones en vuelo.
    await waitFor(() => {
      resolvers.splice(0).forEach((resolve) => resolve());
      // Con el guard solo se marca cada pendiente una vez (lote: '1' y '2').
      expect(mockedPatch).toHaveBeenCalledTimes(2);
    });
    // El clic manual sobre '2' no añadió una segunda petición para ese id.
    expect(mockedPatch.mock.calls.filter(([id]) => id === '2')).toHaveLength(1);
    expect(screen.getByRole('checkbox', { name: /Tomate/ })).toBeChecked();
    expect(screen.getByRole('checkbox', { name: /Lechuga/ })).toBeChecked();
  });

  it('"Marcar todo" sin conexión marca de forma optimista y encola los cambios', async () => {
    vi.stubGlobal('navigator', { ...navigator, vibrate: vi.fn(), onLine: false });
    listState = [
      {
        category: 'Verdulería',
        items: [item({ id: '1', name: 'Tomate' }), item({ id: '2', name: 'Lechuga' })],
      },
    ];
    const user = userEvent.setup();
    renderShopping();

    await screen.findByText('Tomate');
    await user.click(screen.getByRole('button', { name: 'Marcar todo' }));

    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: /Tomate/ })).toBeChecked();
      expect(screen.getByRole('checkbox', { name: /Lechuga/ })).toBeChecked();
    });
    expect(mockedPatch).not.toHaveBeenCalled();
    const alert = await screen.findByTestId('pending-mutations');
    expect(alert).toHaveTextContent('2 cambio(s) pendiente(s) de sincronizar');
  });

  it('"Consolidar desde el menú" pide confirmación antes de mutar', async () => {
    const user = userEvent.setup();
    renderShopping();

    await screen.findByText(/la lista está vacía/i);
    await user.click(screen.getByRole('button', { name: /consolidar desde el menú/i }));

    expect(mockedConsolidate).not.toHaveBeenCalled();
    await user.click(await screen.findByRole('button', { name: 'Consolidar' }));

    await waitFor(() => {
      expect(mockedConsolidate).toHaveBeenCalledTimes(1);
    });
  });

  it('muestra el estado vacío', async () => {
    renderShopping();

    expect(await screen.findByText(/la lista está vacía/i)).toBeInTheDocument();
  });

  it('muestra el error con reintento y recarga la lista', async () => {
    mockedFetchList.mockRejectedValueOnce(new Error('boom'));
    listState = [{ category: 'Verdulería', items: [item({ id: '1', name: 'Tomate' })] }];
    const user = userEvent.setup();
    renderShopping();

    expect(await screen.findByText(/no se pudo cargar la lista/i)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: /reintentar/i }));

    expect(await screen.findByText('Tomate')).toBeInTheDocument();
  });

  it('muestra el encabezado de cada pasillo en el orden recibido', async () => {
    listState = [
      { category: 'Despensa', items: [item({ id: '1', name: 'Arroz', category: 'Despensa' })] },
      { category: 'Verdulería', items: [item({ id: '2', name: 'Tomate' })] },
    ];
    renderShopping();

    await screen.findByRole('heading', { name: /Despensa/ });
    const headings = screen.getAllByRole('heading', { level: 2 }).slice(1);
    expect(within(headings[0]).getByText('Despensa')).toBeInTheDocument();
    expect(within(headings[1]).getByText('Verdulería')).toBeInTheDocument();
  });

  it('permite cambiar la categoría de un ítem desde su fila', async () => {
    listState = [
      { category: 'Otros', items: [item({ id: '1', name: 'Melocotón', category: 'Otros' })] },
    ];
    const user = userEvent.setup();
    renderShopping();

    const selector = await screen.findByRole('combobox', { name: /categoría de melocotón/i });
    const options = within(selector).getAllByRole('option');
    expect(options.map((option) => option.textContent)).toEqual([
      'Verdulería',
      'Pescadería',
      'Carnicería',
      'Refrigerados',
      'Panadería',
      'Bebidas',
      'Despensa',
      'Otros',
    ]);
    expect(selector).toHaveValue('Otros');

    await user.selectOptions(selector, 'Verdulería');

    await waitFor(() => {
      expect(mockedPatch).toHaveBeenCalledWith('1', { category: 'Verdulería' });
    });
  });

  it('muestra el aviso de cambios pendientes cuando hay cola', async () => {
    configureMutationQueue(createMutationStore());
    await enqueueCreate({ name: 'Leche', amount: 1, unit: 'ud' });
    renderShopping();

    const alert = await screen.findByTestId('pending-mutations');
    expect(alert).toHaveTextContent('1 cambio(s) pendiente(s) de sincronizar');
    expect(within(alert).getByRole('button', { name: /sincronizar ahora/i })).toBeEnabled();
  });

  it('expone el encabezado de la vista oculto para lectores de pantalla', () => {
    renderShopping();

    expect(screen.getByRole('heading', { name: 'Lista de la compra' })).toHaveClass(
      'visually-hidden',
    );
  });
});
