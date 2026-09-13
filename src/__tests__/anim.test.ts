import { describe, it, expect } from 'vitest';
import { lerpPosition, upsertKeyframe, easing } from '../lib/anim';

const kfs = [
  { time: 0, x: 0, y: 0 },
  { time: 10, x: 100, y: 40 },
  { time: 20, x: 200, y: 0 },
];

describe('animation keyframe helpers', () => {
  it('clamps before the first and after the last keyframe', () => {
    expect(lerpPosition(kfs, 0)).toEqual({ x: 0, y: 0 });
    expect(lerpPosition(kfs, 25)).toEqual({ x: 200, y: 0 });
  });

  it('linearly interpolates inside a segment', () => {
    expect(lerpPosition(kfs, 5)).toEqual({ x: 50, y: 20 });
    expect(lerpPosition(kfs, 15)).toEqual({ x: 150, y: 20 });
  });

  it('returns null for empty keyframe lists', () => {
    expect(lerpPosition([], 5)).toBeNull();
  });

  it('upserts and keeps the list sorted by time', () => {
    const updated = upsertKeyframe(kfs, { time: 15, x: 150, y: 20 });
    expect(updated.map((k) => k.time)).toEqual([0, 10, 15, 20]);
    // replacing an existing time does not duplicate
    const replaced = upsertKeyframe(kfs, { time: 10, x: 99, y: 1 });
    expect(replaced.filter((k) => k.time === 10)).toHaveLength(1);
    expect(replaced.find((k) => k.time === 10)).toEqual({ time: 10, x: 99, y: 1 });
  });
});

describe('easing', () => {
  it('is identity at endpoints for every curve', () => {
    for (const id of ['linear', 'easeIn', 'easeOut', 'easeInOut', 'back', 'bounce'] as const) {
      expect(easing(id, 0)).toBeCloseTo(0);
      expect(easing(id, 1)).toBeCloseTo(1);
    }
  });

  it('linear is identity, easeIn accelerates', () => {
    expect(easing('linear', 0.5)).toBeCloseTo(0.5);
    expect(easing('easeIn', 0.5)).toBeLessThan(0.5);
    expect(easing('easeOut', 0.5)).toBeGreaterThan(0.5);
  });

  it('lerpPosition applies easing inside a segment', () => {
    // easeIn at midpoint should be under the linear midpoint (150, 20)
    const eased = lerpPosition(kfs, 15, 'easeIn')!;
    expect(eased.x).toBeLessThan(150);
    // linear keeps the old midpoint
    expect(lerpPosition(kfs, 15)!.x).toBe(150);
  });
});
