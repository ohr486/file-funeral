import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Progress } from './progress';

describe('Progress component', () => {
  describe('rendering', () => {
    it('should render a progress bar', () => {
      render(<Progress value={50} aria-label="Loading progress" />);
      const progress = screen.getByRole('progressbar');
      expect(progress).toBeInTheDocument();
    });

    it('should have data-slot attribute', () => {
      render(<Progress value={50} aria-label="Progress" />);
      const progress = screen.getByRole('progressbar');
      expect(progress).toHaveAttribute('data-slot', 'progress');
    });

    it('should render with custom className', () => {
      render(<Progress value={50} className="custom-progress" aria-label="Progress" />);
      const progress = screen.getByRole('progressbar');
      expect(progress).toHaveClass('custom-progress');
    });
  });

  describe('progress values', () => {
    it('should display 0% progress', () => {
      render(<Progress value={0} aria-label="Progress" />);
      const indicator = screen.getByRole('progressbar').querySelector('[data-slot="progress-indicator"]');
      expect(indicator).toHaveStyle({ transform: 'translateX(-100%)' });
    });

    it('should display 25% progress', () => {
      render(<Progress value={25} aria-label="Progress" />);
      const indicator = screen.getByRole('progressbar').querySelector('[data-slot="progress-indicator"]');
      expect(indicator).toHaveStyle({ transform: 'translateX(-75%)' });
    });

    it('should display 50% progress', () => {
      render(<Progress value={50} aria-label="Progress" />);
      const indicator = screen.getByRole('progressbar').querySelector('[data-slot="progress-indicator"]');
      expect(indicator).toHaveStyle({ transform: 'translateX(-50%)' });
    });

    it('should display 75% progress', () => {
      render(<Progress value={75} aria-label="Progress" />);
      const indicator = screen.getByRole('progressbar').querySelector('[data-slot="progress-indicator"]');
      expect(indicator).toHaveStyle({ transform: 'translateX(-25%)' });
    });

    it('should display 100% progress', () => {
      render(<Progress value={100} aria-label="Progress" />);
      const indicator = screen.getByRole('progressbar').querySelector('[data-slot="progress-indicator"]');
      expect(indicator).toHaveStyle({ transform: 'translateX(-0%)' });
    });

    it('should handle undefined value gracefully', () => {
      render(<Progress aria-label="Progress" />);
      const indicator = screen.getByRole('progressbar').querySelector('[data-slot="progress-indicator"]');
      expect(indicator).toHaveStyle({ transform: 'translateX(-100%)' });
    });
  });

  describe('accessibility', () => {
    it('should have progressbar role', () => {
      render(<Progress value={50} aria-label="Loading" />);
      expect(screen.getByRole('progressbar')).toBeInTheDocument();
    });

    it('should support aria-label', () => {
      render(<Progress value={50} aria-label="File upload progress" />);
      expect(screen.getByRole('progressbar', { name: 'File upload progress' })).toBeInTheDocument();
    });

    it('should support aria-valuenow', () => {
      render(<Progress value={60} aria-valuenow={60} aria-label="Progress" />);
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '60');
    });

    it('should support aria-valuemin and aria-valuemax', () => {
      render(
        <Progress
          value={50}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label="Progress"
        />
      );
      const progress = screen.getByRole('progressbar');
      expect(progress).toHaveAttribute('aria-valuemin', '0');
      expect(progress).toHaveAttribute('aria-valuemax', '100');
    });
  });

  describe('indicator', () => {
    it('should render progress indicator', () => {
      render(<Progress value={50} aria-label="Progress" />);
      const indicator = screen.getByRole('progressbar').querySelector('[data-slot="progress-indicator"]');
      expect(indicator).toBeInTheDocument();
    });

    it('should have data-slot attribute on indicator', () => {
      render(<Progress value={50} aria-label="Progress" />);
      const indicator = screen.getByRole('progressbar').querySelector('[data-slot="progress-indicator"]');
      expect(indicator).toHaveAttribute('data-slot', 'progress-indicator');
    });
  });
});
