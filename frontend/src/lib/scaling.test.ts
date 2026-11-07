import { describe, expect, it } from 'vitest';
import { formatScaledAmount, hasRealQuantities, scaleAmount } from './scaling';

describe('scaleAmount', () => {
  it('redondea a entero cuando el resultado es >= 10', () => {
    expect(scaleAmount(500, 0.25)).toBe(125);
    expect(scaleAmount(41, 0.25)).toBe(10); // 10,25 -> 10
    expect(scaleAmount(43, 0.25)).toBe(11); // 10,75 -> 11
  });

  it('redondea a un decimal cuando el resultado está entre 1 y 10', () => {
    expect(scaleAmount(8, 0.25)).toBe(2);
    expect(scaleAmount(7, 0.25)).toBe(1.8); // 1,75 -> 1,8
    expect(scaleAmount(6, 0.25)).toBe(1.5);
  });

  it('redondea a dos decimales cuando el resultado es < 1', () => {
    expect(scaleAmount(1, 0.25)).toBe(0.25);
    expect(scaleAmount(3, 0.25)).toBe(0.75);
    expect(scaleAmount(1, 0.333)).toBe(0.33);
  });

  it('devuelve 0 ante valores no finitos o negativos', () => {
    expect(scaleAmount(Number.NaN, 2)).toBe(0);
    expect(scaleAmount(2, Number.NaN)).toBe(0);
    expect(scaleAmount(Number.POSITIVE_INFINITY, 2)).toBe(0);
    expect(scaleAmount(2, Number.POSITIVE_INFINITY)).toBe(0);
    expect(scaleAmount(-5, 2)).toBe(0);
    expect(scaleAmount(5, -2)).toBe(0);
  });

  it('devuelve 0 cuando el factor es 0', () => {
    expect(scaleAmount(500, 0)).toBe(0);
  });
});

describe('hasRealQuantities', () => {
  it('detecta recetas importadas sin cantidades (todas 1 ud)', () => {
    expect(
      hasRealQuantities([
        { amount: 1, unit: 'ud' },
        { amount: 1, unit: 'ud' },
      ]),
    ).toBe(false);
  });

  it('devuelve true si algún ingrediente tiene cantidad real', () => {
    expect(
      hasRealQuantities([
        { amount: 1, unit: 'ud' },
        { amount: 200, unit: 'g' },
      ]),
    ).toBe(true);
    expect(hasRealQuantities([{ amount: 2, unit: 'ud' }])).toBe(true);
  });

  it('considera un array vacío como con cantidades reales', () => {
    expect(hasRealQuantities([])).toBe(true);
  });
});

describe('formatScaledAmount', () => {
  it('formatea con coma decimal y sin decimales innecesarios', () => {
    expect(formatScaledAmount(1.5)).toBe('1,5');
    expect(formatScaledAmount(2)).toBe('2');
    expect(formatScaledAmount(0.25)).toBe('0,25');
  });

  it('devuelve "0" ante valores no finitos', () => {
    expect(formatScaledAmount(Number.NaN)).toBe('0');
    expect(formatScaledAmount(Number.POSITIVE_INFINITY)).toBe('0');
    expect(formatScaledAmount(Number.NEGATIVE_INFINITY)).toBe('0');
  });
});
