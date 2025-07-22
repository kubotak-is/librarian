import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Button from './Button.svelte';

describe('Button Component', () => {
  it('should render button with text', () => {
    render(Button, { props: { children: 'Click me' } });
    expect(screen.getByRole('button')).toHaveTextContent('Click me');
  });

  it('should apply primary variant styles', () => {
    render(Button, { props: { variant: 'primary', children: 'Primary Button' } });
    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-blue-600');
  });

  it('should apply secondary variant styles', () => {
    render(Button, { props: { variant: 'secondary', children: 'Secondary Button' } });
    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-gray-200');
  });

  it('should apply danger variant styles', () => {
    render(Button, { props: { variant: 'danger', children: 'Danger Button' } });
    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-red-600');
  });

  it('should be disabled when disabled prop is true', () => {
    render(Button, { props: { disabled: true, children: 'Disabled Button' } });
    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
    expect(button).toHaveClass('opacity-50');
  });

  it('should apply small size styles', () => {
    render(Button, { props: { size: 'sm', children: 'Small Button' } });
    const button = screen.getByRole('button');
    expect(button).toHaveClass('px-3', 'py-1.5', 'text-sm');
  });

  it('should apply large size styles', () => {
    render(Button, { props: { size: 'lg', children: 'Large Button' } });
    const button = screen.getByRole('button');
    expect(button).toHaveClass('px-6', 'py-3', 'text-lg');
  });

  it('should show loading state', () => {
    render(Button, { props: { loading: true, children: 'Loading Button' } });
    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
    expect(screen.getByText('読み込み中...')).toBeInTheDocument();
  });

  it('should apply custom class', () => {
    render(Button, { props: { class: 'custom-class', children: 'Custom Button' } });
    const button = screen.getByRole('button');
    expect(button).toHaveClass('custom-class');
  });
});
