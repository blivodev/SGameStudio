export interface SvgShape {
  kind: "rect" | "ellipse" | "star";
  x: number;
  y: number;
  w: number;
  h: number;
  fill: string;
}

export interface SvgStroke {
  points: { x: number; y: number }[];
  color: string;
  size: number;
  eraser: boolean;
}

export interface SvgBone {
  x: number;
  y: number;
  parent: number | null;
}

export interface SvgScene {
  width: number;
  height: number;
  background: string;
  shapes: SvgShape[];
  strokes: SvgStroke[];
  bones: SvgBone[];
}

function starPoints(cx: number, cy: number, r: number, spikes = 5): string {
  const inner = r / 2;
  const pts: string[] = [];
  for (let i = 0; i < spikes * 2; i++) {
    const rad = i % 2 === 0 ? r : inner;
    const a = (i * Math.PI) / spikes - Math.PI / 2;
    pts.push(`${(cx + rad * Math.cos(a)).toFixed(2)},${(cy + rad * Math.sin(a)).toFixed(2)}`);
  }
  return pts.join(" ");
}

function escapeXml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

/**
 * Build a standalone SVG document from the 2D canvas content. This is pure and
 * deterministic so it can be unit-tested and pasted straight into Inkscape/Blender.
 */
export function buildSvg(scene: SvgScene): string {
  const { width, height, background, shapes, strokes, bones } = scene;
  const parts: string[] = [];
  parts.push(`<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">`);
  parts.push(`  <rect x="0" y="0" width="${width}" height="${height}" fill="${escapeXml(background)}" />`);

  // Freehand strokes (eraser strokes skip — they just punch through the background).
  for (const st of strokes) {
    if (st.eraser || st.points.length < 2) continue;
    const pts = st.points.map((p) => `${p.x.toFixed(2)},${p.y.toFixed(2)}`).join(" ");
    parts.push(`  <polyline points="${pts}" fill="none" stroke="${escapeXml(st.color)}" stroke-width="${st.size}" stroke-linecap="round" stroke-linejoin="round" />`);
  }

  // Shapes.
  for (const s of shapes) {
    if (s.kind === "rect") {
      parts.push(`  <rect x="${(s.x - s.w / 2).toFixed(2)}" y="${(s.y - s.h / 2).toFixed(2)}" width="${s.w.toFixed(2)}" height="${s.h.toFixed(2)}" fill="${escapeXml(s.fill)}" stroke="#f0f6fc" stroke-width="1" />`);
    } else if (s.kind === "ellipse") {
      parts.push(`  <ellipse cx="${s.x.toFixed(2)}" cy="${s.y.toFixed(2)}" rx="${(s.w / 2).toFixed(2)}" ry="${(s.h / 2).toFixed(2)}" fill="${escapeXml(s.fill)}" stroke="#f0f6fc" stroke-width="1" />`);
    } else {
      parts.push(`  <polygon points="${starPoints(s.x, s.y, s.w / 2)}" fill="${escapeXml(s.fill)}" stroke="#f0f6fc" stroke-width="1" />`);
    }
  }

  // Skeleton (bones).
  for (let i = 0; i < bones.length; i++) {
    const j = bones[i];
    if (j.parent != null && bones[j.parent]) {
      const p = bones[j.parent];
      parts.push(`  <line x1="${p.x.toFixed(2)}" y1="${p.y.toFixed(2)}" x2="${j.x.toFixed(2)}" y2="${j.y.toFixed(2)}" stroke="#fbbf24" stroke-width="3" stroke-linecap="round" />`);
    }
    parts.push(`  <circle cx="${j.x.toFixed(2)}" cy="${j.y.toFixed(2)}" r="5" fill="#fbbf24" />`);
  }

  parts.push(`</svg>`);
  return parts.join("\n");
}
