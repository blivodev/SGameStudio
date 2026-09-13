export interface Keyframe {
  time: number;
  x: number;
  y: number;
}

export type EasingId = "linear" | "easeIn" | "easeOut" | "easeInOut" | "back" | "bounce";

export const EASING_IDS: EasingId[] = ["linear", "easeIn", "easeOut", "easeInOut", "back", "bounce"];

export const EASING_LABELS: Record<EasingId, string> = {
  linear: "Linear",
  easeIn: "Ease In",
  easeOut: "Ease Out",
  easeInOut: "Ease In-Out",
  back: "Back",
  bounce: "Bounce",
};

/** Apply an easing curve to a normalized 0..1 progress value. */
export function easing(id: EasingId, t: number): number {
  const x = Math.min(1, Math.max(0, t));
  switch (id) {
    case "linear":
      return x;
    case "easeIn":
      return x * x * x;
    case "easeOut":
      return 1 - Math.pow(1 - x, 3);
    case "easeInOut":
      return x < 0.5 ? 4 * x * x * x : 1 - Math.pow(-2 * x + 2, 3) / 2;
    case "back":
      return back(x);
    case "bounce":
      return bounce(x);
  }
}

function back(t: number): number {
  const c1 = 1.70158;
  const c3 = c1 + 1;
  return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2);
}

function bounce(t: number): number {
  const n1 = 7.5625;
  const d1 = 2.75;
  if (t < 1 / d1) return n1 * t * t;
  if (t < 2 / d1) return n1 * (t -= 1.5 / d1) * t + 0.75;
  if (t < 2.5 / d1) return n1 * (t -= 2.25 / d1) * t + 0.9375;
  return n1 * (t -= 2.625 / d1) * t + 0.984375;
}

/**
 * Interpolate position across a sorted keyframe list at time `t`, applying an
 * easing curve within each segment. Returns null for empty lists.
 */
export function lerpPosition(keyframes: Keyframe[], t: number, easeId: EasingId = "linear"): { x: number; y: number } | null {
  if (!keyframes || keyframes.length === 0) return null;
  if (t <= keyframes[0].time) return { x: keyframes[0].x, y: keyframes[0].y };
  const last = keyframes[keyframes.length - 1];
  if (t >= last.time) return { x: last.x, y: last.y };
  for (let i = 0; i < keyframes.length - 1; i++) {
    const a = keyframes[i];
    const b = keyframes[i + 1];
    if (t >= a.time && t <= b.time) {
      const span = b.time - a.time || 1;
      const f = easing(easeId, (t - a.time) / span);
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
