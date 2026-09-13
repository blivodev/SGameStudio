import { describe, it, expect } from 'vitest';
import { buildSvg } from '../lib/svg';

const scene = {
  width: 800,
  height: 600,
  background: '#0d1117',
  shapes: [
    { kind: 'rect' as const, x: 100, y: 100, w: 40, h: 20, fill: '#7c3aed' },
    { kind: 'ellipse' as const, x: 200, y: 150, w: 60, h: 40, fill: '#22d3ee' },
    { kind: 'star' as const, x: 300, y: 120, w: 50, h: 50, fill: '#f43f5e' },
  ],
  strokes: [
    { points: [{ x: 0, y: 0 }, { x: 10, y: 10 }], color: '#34d399', size: 6, eraser: false },
    { points: [{ x: 1, y: 1 }, { x: 2, y: 2 }], color: '#000', size: 6, eraser: true },
  ],
  bones: [
    { x: 50, y: 50, parent: null },
    { x: 80, y: 50, parent: 0 },
  ],
};

describe('SVG builder', () => {
  it('produces a self-contained SVG document', () => {
    const svg = buildSvg(scene);
    expect(svg.startsWith('<svg xmlns="http://www.w3.org/2000/svg"')).toBe(true);
    expect(svg).toContain('viewBox="0 0 800 600"');
    expect(svg.endsWith('</svg>')).toBe(true);
  });

  it('emits a background rect and each shape type', () => {
    const svg = buildSvg(scene);
    expect(svg).toContain('fill="#0d1117"');
    expect(svg).toContain('<rect');
    expect(svg).toContain('<ellipse');
    expect(svg).toContain('<polygon');
  });

  it('emits non-eraser strokes as polylines and skips eraser strokes', () => {
    const svg = buildSvg(scene);
    const polylineCount = (svg.match(/<polyline/g) ?? []).length;
    // Only the brush stroke (green) should appear; the eraser stroke is skipped.
    expect(polylineCount).toBe(1);
    expect(svg).toContain('#34d399');
  });

  it('emits bones as lines + joint circles', () => {
    const svg = buildSvg(scene);
    expect(svg).toContain('<line');
    expect(svg).toContain('stroke="#fbbf24"');
    expect((svg.match(/<circle/g) ?? []).length).toBe(2);
  });
});
