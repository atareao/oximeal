import { describe, expect, it, vi, afterEach } from 'vitest';
import {
  addDays,
  formatDayLabel,
  formatWeekRange,
  getWeekDays,
  isValidDateString,
  isToday,
  startOfWeek,
  toDateString,
} from './dates';

describe('startOfWeek', () => {
  it('devuelve el lunes de la semana para un miércoles', () => {
    expect(startOfWeek('2026-11-04')).toBe('2026-11-02');
  });

  it('devuelve el mismo lunes cuando la fecha ya es lunes', () => {
    expect(startOfWeek('2026-11-02')).toBe('2026-11-02');
  });

  it('devuelve el lunes anterior para un domingo', () => {
    expect(startOfWeek('2026-11-08')).toBe('2026-11-02');
  });

  it('el resultado siempre es lunes', () => {
    for (let day = 1; day <= 30; day += 1) {
      const date = `2026-06-${String(day).padStart(2, '0')}`;
      const monday = startOfWeek(date);
      const parsed = new Date(`${monday}T00:00:00`);
      expect(parsed.getDay()).toBe(1);
    }
  });

  it('cruza el cambio de mes', () => {
    expect(startOfWeek('2026-03-01')).toBe('2026-02-23');
  });

  it('cruza el cambio de año', () => {
    expect(startOfWeek('2026-01-01')).toBe('2025-12-29');
  });

  it('acepta un Date y no sufre desfase de zona', () => {
    expect(startOfWeek(new Date(2026, 10, 4))).toBe('2026-11-02');
  });
});

describe('addDays', () => {
  it('suma días dentro del mismo mes', () => {
    expect(addDays('2026-11-02', 3)).toBe('2026-11-05');
  });

  it('resta días cruzando de mes', () => {
    expect(addDays('2026-11-02', -3)).toBe('2026-10-30');
  });

  it('cruza el cambio de año', () => {
    expect(addDays('2025-12-31', 1)).toBe('2026-01-01');
  });

  it('soporta años bisiestos', () => {
    expect(addDays('2028-02-28', 1)).toBe('2028-02-29');
  });
});

describe('toDateString', () => {
  it('formatea en local sin desfase UTC', () => {
    expect(toDateString(new Date(2026, 0, 1))).toBe('2026-01-01');
  });
});

describe('getWeekDays', () => {
  it('devuelve siete días desde el lunes', () => {
    expect(getWeekDays('2026-11-02')).toEqual([
      '2026-11-02',
      '2026-11-03',
      '2026-11-04',
      '2026-11-05',
      '2026-11-06',
      '2026-11-07',
      '2026-11-08',
    ]);
  });
});

describe('formatDayLabel', () => {
  it('devuelve día abreviado y número', () => {
    expect(formatDayLabel('2026-11-02')).toBe('Lun 2');
    expect(formatDayLabel('2026-11-08')).toBe('Dom 8');
  });
});

describe('formatWeekRange', () => {
  it('formatea un rango dentro del mismo mes', () => {
    expect(formatWeekRange('2026-11-02', '2026-11-08')).toBe('2 – 8 nov 2026');
  });

  it('formatea un rango que cruza de mes', () => {
    expect(formatWeekRange('2026-11-30', '2026-12-06')).toBe('30 nov – 6 dic 2026');
  });

  it('formatea un rango que cruza de año', () => {
    expect(formatWeekRange('2025-12-29', '2026-01-04')).toBe('29 dic 2025 – 4 ene 2026');
  });
});

describe('isToday', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('devuelve true para la fecha actual', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 10, 4, 12, 0, 0));
    expect(isToday('2026-11-04')).toBe(true);
  });

  it('devuelve false para otra fecha', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 10, 4, 12, 0, 0));
    expect(isToday('2026-11-05')).toBe(false);
  });
});

describe('isValidDateString', () => {
  it('rechaza fechas imposibles', () => {
    expect(isValidDateString('2026-13-45')).toBe(false);
    expect(isValidDateString('2026-02-29')).toBe(false);
  });

  it('acepta fechas reales, incluidas las de año bisiesto', () => {
    expect(isValidDateString('2028-02-29')).toBe(true);
    expect(isValidDateString('2026-11-02')).toBe(true);
  });

  it('rechaza formatos que no son YYYY-MM-DD', () => {
    expect(isValidDateString('2026-1-2')).toBe(false);
    expect(isValidDateString('hoy')).toBe(false);
  });
});
