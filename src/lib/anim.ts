export interface Keyframe {
  time: number;
  x: number;
  y: number;
}

/** Linear position interpolation across a sorted keyframe list at time `t`. */
export function lerpPosition(keyframes: Keyframe[], t: number): { x: number; y: number } | null {
  if (!keyframes || keyframes.length === 0) return null;
  if (t <= keyframes[0].time) return { x: keyframes[0].x, y: keyframes[0].y };
  const last = keyframes[keyframes.length - 1];
  if (t >= last.time) return { x: last.x, y: last.y };
  for (let i = 0; i < keyframes.length - 1; i++) {
    const a = keyframes[i];
    const b = keyframes[i + 1];
    if (t >= a.time && t <= b.time) {
      const span = b.time - a.time || 1;
      const f = (t - a.time) / span;
      return { x: a.x + (b.x - a.x) * f, y: a.y + (b.y - a.y) * f };
    }
  }
  return null;
}

/** Insert/replace a keyframe, keeping the list sorted by time. */
export function upsertKeyframe(list: Keyframe[], kf: Keyframe): Keyframe[] {
  const replaced = list.filter((k) => k.time !== kf.time);
  replaced.push(kf);
  replaced.sort((a, b) => a.time - b.time);
  return replaced;
}
