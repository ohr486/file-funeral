import { describe, it, expect } from 'vitest';
import { cn } from './utils';

describe('cn utility function', () => {
  it('should merge multiple class names', () => {
    const result = cn('text-red-500', 'bg-blue-500');
    expect(result).toBe('text-red-500 bg-blue-500');
  });

  it('should handle conditional classes with objects', () => {
    const result = cn({
      'text-red-500': true,
      'bg-blue-500': false,
      'font-bold': true,
    });
    expect(result).toBe('text-red-500 font-bold');
  });

  it('should handle arrays of class names', () => {
    const result = cn(['text-red-500', 'bg-blue-500']);
    expect(result).toBe('text-red-500 bg-blue-500');
  });

  it('should merge conflicting Tailwind classes correctly', () => {
    // tailwind-merge should keep the last conflicting class
    const result = cn('px-2 py-1', 'px-4');
    expect(result).toBe('py-1 px-4');
  });

  it('should handle undefined and null values', () => {
    const result = cn('text-red-500', undefined, null, 'bg-blue-500');
    expect(result).toBe('text-red-500 bg-blue-500');
  });

  it('should handle empty strings', () => {
    const result = cn('', 'text-red-500', '');
    expect(result).toBe('text-red-500');
  });

  it('should merge complex Tailwind utility combinations', () => {
    // When both classes modify the same property, the last one wins
    const result = cn('text-sm font-normal', 'text-lg font-bold');
    expect(result).toBe('text-lg font-bold');
  });

  it('should handle mixed input types', () => {
    const result = cn(
      'base-class',
      { 'conditional-class': true, 'hidden-class': false },
      ['array-class'],
      undefined,
      'final-class'
    );
    expect(result).toBe('base-class conditional-class array-class final-class');
  });

  it('should return empty string when no arguments provided', () => {
    const result = cn();
    expect(result).toBe('');
  });

  it('should properly merge responsive and state variants', () => {
    const result = cn('hover:bg-blue-500', 'hover:bg-red-500');
    expect(result).toBe('hover:bg-red-500');
  });
});
