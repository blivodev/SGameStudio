import { describe, it, expect } from 'vitest';
import { lerpPosition, lerpPosition3, upsertKeyframe, upsertKeyframe3, easing } from '../lib/anim';

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

describe('3D pose interpolation', () => {
  const k3 = [
    { time: 0, x: 0, y: 0, z: 0 },
    { time: 10, x: 10, y: 20, z: 30 },
    { time: 20, x: 20, y: 40, z: 0 },
  ];

  it('interpolates xyz linearly across a segment', () => {
    expect(lerpPosition3(k3, 5)).toEqual({ x: 5, y: 10, z: 15 });
    expect(lerpPosition3(k3, 15)).toEqual({ x: 15, y: 30, z: 15 });
  });

  it('clamps to endpoints and handles empty lists', () => {
    expect(lerpPosition3(k3, 0)).toEqual({ x: 0, y: 0, z: 0 });
    expect(lerpPosition3(k3, 30)).toEqual({ x: 20, y: 40, z: 0 });
    expect(lerpPosition3([], 5)).toBeNull();
  });

  it('upserts and keeps 3D poses sorted', () => {
    const updated = upsertKeyframe3(k3, { time: 15, x: 99, y: 1, z: 2 });
    expect(updated.map((k) => k.time)).toEqual([0, 10, 15, 20]);
    expect(updated.find((k) => k.time === 15)).toEqual({ time: 15, x: 99, y: 1, z: 2 });
  });
});
