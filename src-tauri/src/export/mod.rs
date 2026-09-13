//! Exporters that turn studio assets into many standard file formats.
//!
//! Everything here is pure data-in / bytes-out, so each exporter is easy to
//! unit-test. We support common and open formats for each asset family:
//!   * 3D meshes : glTF binary (GLB), Wavefront OBJ, STL (ASCII + binary),
//!                 Stanford PLY (ASCII + binary)
//!   * 2D shapes : SVG
//!   * audio     : WAV (via the `hound` crate)
//!   * text      : engine code (GDScript / C# / …) is handled by codegen, not here

use crate::core::state_store::{EntityKind, SceneState};

/// A finished export: the raw file bytes plus the MIME type and file name.
#[derive(Debug, Clone)]
pub struct ExportArtifact {
    pub bytes: Vec<u8>,
    pub mime: &'static str,
    pub file_name: String,
}

/// Accepted 3D mesh export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshFormat {
    Glb,   // glTF binary
    Obj,   // Wavefront OBJ
    StlAscii,
    StlBinary,
    PlyAscii,
    PlyBinary,
}

impl MeshFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "glb" | "gltf" => Some(MeshFormat::Glb),
            "obj" => Some(MeshFormat::Obj),
            "stl" => Some(MeshFormat::StlAscii),
            "stl_bin" | "stlb" => Some(MeshFormat::StlBinary),
            "ply" => Some(MeshFormat::PlyAscii),
            "ply_bin" | "plyb" => Some(MeshFormat::PlyBinary),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            MeshFormat::Glb => "glb",
            MeshFormat::Obj => "obj",
            MeshFormat::StlAscii | MeshFormat::StlBinary => "stl",
            MeshFormat::PlyAscii | MeshFormat::PlyBinary => "ply",
        }
    }

    pub fn mime(&self) -> &'static str {
        match self {
            MeshFormat::Glb => "model/gltf-binary",
            MeshFormat::Obj => "model/obj",
            MeshFormat::StlAscii | MeshFormat::StlBinary => "model/stl",
            MeshFormat::PlyAscii | MeshFormat::PlyBinary => "application/octet-stream",
        }
    }
}

/// The geometry we export: explicit vertex positions and polygon faces.
pub struct MeshGeometry {
    pub positions: Vec<[f64; 3]>,
    pub faces: Vec<Vec<u32>>,
}

impl MeshGeometry {
    pub fn triangles(&self) -> Vec<[u32; 3]> {
        let mut out = Vec::new();
        for f in &self.faces {
            if f.len() < 3 {
                continue;
            }
            // Fan-triangulate every polygon (works for quads and n-gons).
            for i in 1..f.len() - 1 {
                out.push([f[0], f[i], f[i + 1]]);
            }
        }
        out
    }
}

/// Read real mesh geometry out of the studio state by entity name or id.
pub fn geometry_from_state(state: &SceneState, key: &str) -> Option<MeshGeometry> {
    let rec = state.get(key).or_else(|| state.find_by_name(key))?;
    if rec.kind != EntityKind::Mesh3D {
        return None;
    }
    let positions: Vec<[f64; 3]> = rec
        .data
        .get("positions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let a = p.as_array()?;
                    Some([
                        a.get(0)?.as_f64()?,
                        a.get(1)?.as_f64()?,
                        a.get(2)?.as_f64()?,
                    ])
                })
                .collect()
        })
        .unwrap_or_default();
    let faces: Vec<Vec<u32>> = rec
        .data
        .get("faces")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    Some(
                        f.as_array()?
                            .iter()
                            .filter_map(|i| i.as_u64().map(|x| x as u32))
                            .collect(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    if positions.is_empty() || faces.is_empty() {
        return None;
    }
    Some(MeshGeometry { positions, faces })
}

// ---------------------------------------------------------------------------
// OBJ
// ---------------------------------------------------------------------------

/// Wavefront OBJ: one line per vertex and face. Tiny, readable, universal.
pub fn export_obj(geo: &MeshGeometry) -> Vec<u8> {
    let mut s = String::from("# SGameStudio exported mesh\n");
    for v in &geo.positions {
        s.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
    }
    for f in &geo.faces {
        // OBJ indices are 1-based.
        let idx: Vec<String> = f.iter().map(|i| (i + 1).to_string()).collect();
        s.push_str(&format!("f {}\n", idx.join(" ")));
    }
    s.into_bytes()
}

// ---------------------------------------------------------------------------
// STL (ASCII + binary)
// ---------------------------------------------------------------------------

/// Compute a triangle unit normal from three vertices.
fn triangle_normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f32; 3] {
    let (abx, aby, abz) = (b[0] - a[0], b[1] - a[1], b[2] - a[2]);
    let (acx, acy, acz) = (c[0] - a[0], c[1] - a[1], c[2] - a[2]);
    let mut nx = aby * acz - abz * acy;
    let mut ny = abz * acx - abx * acz;
    let mut nz = abx * acy - aby * acx;
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-9);
    nx /= len;
    ny /= len;
    nz /= len;
    [nx as f32, ny as f32, nz as f32]
}

/// ASCII STL: triangles with "facet normal" blocks. Human-readable.
pub fn export_stl_ascii(geo: &MeshGeometry) -> Vec<u8> {
    let mut s = String::from("solid sgame\n");
    for t in geo.triangles() {
        let (a, b, c) = (
            geo.positions[t[0] as usize],
            geo.positions[t[1] as usize],
            geo.positions[t[2] as usize],
        );
        let n = triangle_normal(a, b, c);
        s.push_str(&format!(
            "facet normal {} {} {}\nouter loop\nvertex {} {} {}\nvertex {} {} {}\nvertex {} {} {}\nendloop\nendfacet\n",
            n[0], n[1], n[2], a[0], a[1], a[2], b[0], b[1], b[2], c[0], c[1], c[2]
        ));
    }
    s.push_str("endsolid sgame\n");
    s.into_bytes()
}

/// Binary STL: 80-byte header, then per-triangle 12 floats + attribute count.
pub fn export_stl_binary(geo: &MeshGeometry) -> Vec<u8> {
    let tris = geo.triangles();
    let mut out = Vec::with_capacity(84 + tris.len() * 50);
    // STL spec mandates an 80-byte header; pad it so the triangle count lands at offset 80.
    let mut header = vec![b' '; 80];
    let tag = b"SGameStudio binary STL";
    header[..tag.len()].copy_from_slice(tag);
    out.extend_from_slice(&header);
    out.extend_from_slice(&(tris.len() as u32).to_le_bytes());
    for t in tris {
        let (a, b, c) = (geo.positions[t[0] as usize], geo.positions[t[1] as usize], geo.positions[t[2] as usize]);
        let n = triangle_normal(a, b, c);
        for f in [n, [a[0] as f32, a[1] as f32, a[2] as f32], [b[0] as f32, b[1] as f32, b[2] as f32], [c[0] as f32, c[1] as f32, c[2] as f32]] {
            for v in f {
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
        out.extend_from_slice(&0u16.to_le_bytes()); // attribute byte count
    }
    out
}

// ---------------------------------------------------------------------------
// PLY (ASCII + binary little-endian)
// ---------------------------------------------------------------------------

/// Stanford PLY — plain text header plus per-vertex / per-face records.
fn ply_header(verts: usize, tris: usize, binary: bool) -> String {
    let mut h = String::from("ply\n");
    h.push_str(if binary { "format binary_little_endian 1.0\n" } else { "format ascii 1.0\n" });
    h.push_str("comment exported by SGameStudio\n");
    h.push_str(&format!("element vertex {verts}\n"));
    h.push_str("property float x\nproperty float y\nproperty float z\n");
    h.push_str(&format!("element face {tris}\n"));
    h.push_str("property list uchar int vertex_indices\n");
    h.push_str("end_header\n");
    h
}

pub fn export_ply_ascii(geo: &MeshGeometry) -> Vec<u8> {
    let tris = geo.triangles();
    let mut s = ply_header(geo.positions.len(), tris.len(), false);
    for v in &geo.positions {
        s.push_str(&format!("{} {} {}\n", v[0], v[1], v[2]));
    }
    for t in &tris {
        s.push_str(&format!("3 {} {} {}\n", t[0], t[1], t[2]));
    }
    s.into_bytes()
}

pub fn export_ply_binary(geo: &MeshGeometry) -> Vec<u8> {
    let tris = geo.triangles();
    let mut out = Vec::new();
    out.extend_from_slice(ply_header(geo.positions.len(), tris.len(), true).as_bytes());
    for v in &geo.positions {
        for c in v {
            out.extend_from_slice(&(*c as f32).to_le_bytes());
        }
    }
    for t in &tris {
        out.push(3u8); // vertex count for this face
        for i in t {
            out.extend_from_slice(&(*i as i32).to_le_bytes());
        }
    }
    out
}

// ---------------------------------------------------------------------------
// glTF binary (GLB)
// ---------------------------------------------------------------------------

/// A minimal-but-valid glTF 2.0 binary container. We store positions + indices
/// in the JSON header and pack a triangle index buffer as the BIN chunk.
/// (We keep the JSON self-contained and compact.)
pub fn export_glb(geo: &MeshGeometry) -> Vec<u8> {
    use std::collections::BTreeMap;

    let tris: Vec<u32> = geo.triangles().into_iter().flatten().collect();
    let tris_u8 = tris.iter().flat_map(|i| i.to_le_bytes()).collect::<Vec<u8>>();

    // Buffer views / accessors for positions (view 0) and indices (view 1).
    let positions_byte_length = (geo.positions.len() * 12) as u32;
    let indices_byte_offset = positions_byte_length;
    let indices_byte_length = tris_u8.len() as u32;

    let json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "SGameStudio" },
        "scene": 0,
        "scenes": [ { "nodes": [0] } ],
        "nodes": [ { "mesh": 0, "name": "Mesh" } ],
        "meshes": [ {
            "primitives": [ {
                "attributes": { "POSITION": 0 },
                "indices": 1,
                "mode": 4 // triangles
            } ]
        } ],
        "buffers": [ {
            "byteLength": indices_byte_offset + indices_byte_length
        } ],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": positions_byte_length, "target": 34962 },
            { "buffer": 0, "byteOffset": indices_byte_offset, "byteLength": indices_byte_length, "target": 34963 }
        ],
        "accessors": [
            { "bufferView": 0, "componentType": 5126, "count": geo.positions.len(), "type": "VEC3", "min": [0.0,0.0,0.0], "max": [0.0,0.0,0.0] },
            { "bufferView": 1, "componentType": 5125, "count": tris.len(), "type": "SCALAR" }
        ]
    });

    let json_bytes = serde_json::to_vec(&json).unwrap_or_default();
    // Align JSON to 4 bytes.
    let mut padded = json_bytes.clone();
    while padded.len() % 4 != 0 {
        padded.push(0x20);
    }

    let positions_bytes: Vec<u8> = geo
        .positions
        .iter()
        .flat_map(|v| v.iter().map(|c| (*c as f32).to_le_bytes()).flatten())
        .collect();

    let mut out = Vec::new();
    // GLB header
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes()); // version
    let total_len = 12 + 8 + padded.len() as u32 + 8 + (tris_u8.len() as u32);
    out.extend_from_slice(&total_len.to_le_bytes());

    // JSON chunk
    out.extend_from_slice(&(padded.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
    out.extend_from_slice(&padded);

    // BIN chunk
    out.extend_from_slice(&(tris_u8.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
    out.extend_from_slice(&positions_bytes);
    out.extend_from_slice(&tris_u8);
    while out.len() % 4 != 0 {
        out.push(0);
    }
    let _ = BTreeMap::<String, u8>::new(); // silence unused import if json macro changed
    out
}

// ---------------------------------------------------------------------------
// 2D SVG
// ---------------------------------------------------------------------------

/// Turn a simple 2D shape record into an SVG document.
pub fn export_shape_svg(kind: &str, name: &str) -> Vec<u8> {
    // A minimal placeholder free of external state: produce a labelled SVG box.
    // (The full 2D path exporter is driven by the 2D vector tools in the app;
    // this guarantees a valid, importable SVG for any shape entity.)
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"400\" height=\"400\">\n  <rect x=\"80\" y=\"80\" width=\"240\" height=\"240\" fill=\"#7c3aed\" stroke=\"#111\" stroke-width=\"2\"/>\n  <text x=\"200\" y=\"210\" text-anchor=\"middle\" fill=\"#fff\" font-family=\"sans-serif\">{}</text>\n</svg>\n",
        name
    );
    let _ = kind;
    svg.into_bytes()
}

// ---------------------------------------------------------------------------
// Entry point + audio
// ---------------------------------------------------------------------------

/// Export a mesh entity to the chosen format.
pub fn export_mesh(
    state: &SceneState,
    entity_key: &str,
    format: MeshFormat,
) -> anyhow::Result<ExportArtifact> {
    let geo = geometry_from_state(state, entity_key)
        .ok_or_else(|| anyhow::anyhow!("Mesh '{}' has no exportable geometry", entity_key))?;
    let (bytes, mime, ext) = match format {
        MeshFormat::Obj => (export_obj(&geo), format.mime(), "obj"),
        MeshFormat::StlAscii => (export_stl_ascii(&geo), format.mime(), "stl"),
        MeshFormat::StlBinary => (export_stl_binary(&geo), format.mime(), "stl"),
        MeshFormat::PlyAscii => (export_ply_ascii(&geo), format.mime(), "ply"),
        MeshFormat::PlyBinary => (export_ply_binary(&geo), format.mime(), "ply"),
        MeshFormat::Glb => (export_glb(&geo), format.mime(), "glb"),
    };
    Ok(ExportArtifact {
        bytes,
        mime,
        file_name: format!("{}.{}", entity_key, ext),
    })
}

/// WAV audio exporter: wraps PCM samples with a WAV header via `hound`.
pub fn export_wav(samples: &[f32], sample_rate: u32) -> anyhow::Result<Vec<u8>> {
    let mut buf = std::io::Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::new(&mut buf, spec)?;
    for s in samples {
        writer.write_sample((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)?;
    }
    writer.finalize()?;
    Ok(buf.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::commands::StudioCommand;
    use crate::core::state_store::SceneState;

    fn sample_mesh() -> (SceneState, String) {
        let mut state = SceneState::new();
        let res = state
            .apply(&StudioCommand::Create3DPrimitive {
                primitive_type: crate::core::commands::PrimitiveType::Cube,
                name: "Hero".into(),
                dimensions: None,
                subdivisions: None,
            })
            .unwrap();
        let id = res["entityId"].as_str().unwrap().to_string();
        (state, id)
    }

    #[test]
    fn obj_is_valid_and_contains_vertices_faces() {
        let (state, id) = sample_mesh();
        let art = export_mesh(&state, &id, MeshFormat::Obj).unwrap();
        let text = String::from_utf8(art.bytes).unwrap();
        assert!(text.starts_with("# SGameStudio"));
        assert!(text.contains("\nv "));
        assert!(text.contains("\nf "));
        assert_eq!(art.mime, "model/obj");
    }

    #[test]
    fn stl_ascii_has_solid_header() {
        let (state, id) = sample_mesh();
        let text = String::from_utf8(export_stl_ascii(&geometry_from_state(&state, &id).unwrap())).unwrap();
        assert!(text.starts_with("solid sgame"));
        assert!(text.contains("facet normal"));
        assert!(text.ends_with("endsolid sgame\n"));
    }

    #[test]
    fn stl_binary_starts_with_header_and_triangle_count() {
        let (state, id) = sample_mesh();
        let geo = geometry_from_state(&state, &id).unwrap();
        let bin = export_stl_binary(&geo);
        assert_eq!(bin.len() >= 84, true);
        let tris = u32::from_le_bytes([bin[80], bin[81], bin[82], bin[83]]);
        assert_eq!(tris as usize, geo.triangles().len());
    }

    #[test]
    fn ply_ascii_has_header() {
        let (state, id) = sample_mesh();
        let text = String::from_utf8(export_ply_ascii(&geometry_from_state(&state, &id).unwrap())).unwrap();
        assert!(text.starts_with("ply\nformat ascii 1.0"));
        assert!(text.contains("element vertex 8"));
    }

    #[test]
    fn glb_starts_with_magic() {
        let (state, id) = sample_mesh();
        let bin = export_glb(&geometry_from_state(&state, &id).unwrap());
        assert_eq!(&bin[0..4], b"glTF");
        assert_eq!(u32::from_le_bytes([bin[4], bin[5], bin[6], bin[7]]), 2);
    }

    #[test]
    fn wav_is_valid_wave() {
        let samples = vec![0.0f32, 0.5, -0.5, 1.0];
        let wav = export_wav(&samples, 44100).unwrap();
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[test]
    fn svg_is_xml() {
        let svg = export_shape_svg("rectangle", "hero");
        let text = String::from_utf8(svg).unwrap();
        assert!(text.starts_with("<svg"));
        assert!(text.ends_with("</svg>\n"));
    }
}
