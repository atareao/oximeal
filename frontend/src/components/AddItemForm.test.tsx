import { describe, expect, it, vi, beforeEach } from 'vitest';
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App as AntdApp } from 'antd';
import { useAddShoppingItem } from '../api/hooks';
import { AddItemForm } from './AddItemForm';

vi.mock('../api/hooks', () => ({
  useAddShoppingItem: vi.fn(),
}));

const mockedUseAdd = vi.mocked(useAddShoppingItem);
const addMutate = vi.fn();

type MutateOptions = { onSuccess?: () => void; onError?: (error: unknown) => void };

function renderForm() {
  return render(
    <AntdApp>
      <AddItemForm />
    </AntdApp>,
  );
}

function form() {
  return screen.getByRole('form', { name: /añadir ítem/i });
}

describe('AddItemForm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedUseAdd.mockReturnValue({
      mutate: addMutate,
      isPending: false,
      isError: false,
      isSuccess: false,
      isIdle: true,
      reset: vi.fn(),
      status: 'idle',
      variables: undefined,
      data: undefined,
      error: null,
      context: undefined,
      failureCount: 0,
      failureReason: null,
      isPaused: false,
      submittedAt: 0,
    } as unknown as ReturnType<typeof useAddShoppingItem>);
  });

  it('avisa y no muta si el nombre está vacío', async () => {
    renderForm();

    fireEvent.submit(form());

    expect(await screen.findByText('Escribe el nombre del producto')).toBeInTheDocument();
    expect(addMutate).not.toHaveBeenCalled();
  });

  it('avisa y no muta si la cantidad es cero', async () => {
    const user = userEvent.setup();
    renderForm();

    await user.type(screen.getByLabelText(/nombre del producto/i), 'Leche');
    const amount = screen.getByLabelText(/cantidad/i);
    await user.clear(amount);
    await user.type(amount, '0');

    fireEvent.submit(form());

    expect(await screen.findByText('La cantidad debe ser mayor que cero')).toBeInTheDocument();
    expect(addMutate).not.toHaveBeenCalled();
  });

  it('acepta la cantidad con coma y la envía como número decimal', async () => {
    const user = userEvent.setup();
    renderForm();

    await user.type(screen.getByLabelText(/nombre del producto/i), 'Leche');
    const amount = screen.getByLabelText(/cantidad/i) as HTMLInputElement;
    // El input es `type="number"`, que descarta la coma; forzamos el valor para
    // ejercer la normalización `1,5 → 1.5` de la lógica del formulario.
    amount.type = 'text';
    fireEvent.change(amount, { target: { value: '1,5' } });

    fireEvent.submit(form());

    await waitFor(() => {
      expect(addMutate).toHaveBeenCalledWith(
        { name: 'Leche', amount: 1.5, unit: 'ud' },
        expect.anything(),
      );
    });
  });

  it('no borra lo escrito si el usuario teclea el siguiente ítem durante la petición', async () => {
    let captured: MutateOptions | undefined;
    addMutate.mockImplementation((_input: unknown, options?: MutateOptions) => {
      captured = options;
    });
    const user = userEvent.setup();
    renderForm();

    const name = screen.getByLabelText(/nombre del producto/i);
    await user.type(name, 'Leche');
    fireEvent.submit(form());

    expect(addMutate).toHaveBeenCalledTimes(1);

    // Mientras la petición está en vuelo el usuario prepara el siguiente ítem.
    await user.clear(name);
    await user.type(name, 'Pan');

    // Llega la respuesta de la petición anterior.
    act(() => {
      captured?.onSuccess?.();
    });

    expect(name).toHaveValue('Pan');
  });
});
