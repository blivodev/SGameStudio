use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexId(pub u32);
pub struct FaceId(pub u32);
pub struct HalfEdgeId(pub u32);

#[derive(Debug, Error)]
pub enum MeshError {
    #[error("unknown face index {0}")]
    UnknownFace(usize),
    #[error("mesh is not manifold: {0}")]
    NonManifold(String),
    #[error("operation not supported yet: {0}")]
    Unsupported(&'static str),
}

#[derive(Debug, Clone, Default)]
pub struct HalfEdgeMesh {
    positions: Vec<[f64; 3]>,
    faces: Vec<Vec<u32>>,
    half_edges: Vec<HalfEdge>,
    is_dirty: bool,
}

#[derive(Debug, Clone, Copy)]
struct HalfEdge {
    twin: u32,
    next: u32,
    origin: u32,
    face: u32,
}

impl HalfEdgeMesh {
    pub fn new() -> Self {
        HalfEdgeMesh {
            positions: Vec::new(),
            faces: Vec::new(),
            half_edges: Vec::new(),
            is_dirty: true,
        }
    }

    pub fn from_faces(positions: Vec<[f64; 3]>, faces: Vec<Vec<u32>>) -> Result<Self, MeshError> {
        for face in &faces {
            if face.len() < 3 {
                return Err(MeshError::NonManifold("degenerate face".to_string()));
            }
        }

        let mut mesh = HalfEdgeMesh {
            positions,
            faces,
            half_edges: Vec::new(),
            is_dirty: true,
        };

        mesh.rebuild_half_edges()?;
        Ok(mesh)
    }

    fn rebuild_half_edges(&mut self) -> Result<(), MeshError> {
        self.half_edges.clear();
        // Map each directed edge (origin, target) to the index of its half-edge.
        let mut edge_index: HashMap<(u32, u32), u32> = HashMap::new();

        // Pass 1: build one half-edge per directed edge, grouped by face.
        for (face_idx, face) in self.faces.iter().enumerate() {
            let n = face.len();
            let start = self.half_edges.len() as u32;
            for i in 0..n {
                let a = face[i];
                let b = face[(i + 1) % n];
                let index = self.half_edges.len() as u32;
                self.half_edges.push(HalfEdge {
                    twin: u32::MAX,
                    next: 0,
                    origin: a,
                    face: face_idx as u32,
                });
                edge_index.insert((a, b), index);
            }
            // Wire the `next` pointers into a closed ring around the face.
            for i in 0..n {
                self.half_edges[(start + i as u32) as usize].next = start + ((i + 1) % n) as u32;
            }
        }

        // Pass 2: resolve twins by matching each directed edge to its reverse.
        let he_count = self.half_edges.len();
        for i in 0..he_count {
            let a = self.half_edges[i].origin;
            let b = self.half_edges[self.half_edges[i].next as usize].origin;
            let Some(&twin) = edge_index.get(&(b, a)) else {
                return Err(MeshError::NonManifold("open edge".to_string()));
            };
            if twin == i as u32 {
                return Err(MeshError::NonManifold("non-manifold edge".to_string()));
            }
            self.half_edges[i].twin = twin;
        }

        // Verify twin symmetry.
        for i in 0..he_count {
            let twin = self.half_edges[i].twin as usize;
            if twin >= he_count || self.half_edges[twin].twin != i as u32 {
                return Err(MeshError::NonManifold("invalid twin".to_string()));
            }
        }

        self.is_dirty = false;
        Ok(())
    }

    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    pub fn edge_count(&self) -> usize {
        self.half_edges.len() / 2
    }

    pub fn is_manifold(&mut self) -> bool {
        if self.is_dirty {
            self.rebuild_half_edges().is_ok()
        } else {
            true
        }
    }

    pub fn euler_characteristic(&self) -> i32 {
        (self.vertex_count() as i32) - (self.edge_count() as i32) + (self.face_count() as i32)
    }

    pub fn positions(&self) -> &[[f64; 3]] {
        &self.positions
    }

    /// The list of faces, each being the vertex indices that make up one polygon.
    pub fn faces(&self) -> &[Vec<u32>] {
        &self.faces
    }

    pub fn triangle_indices(&self) -> Vec<[u32; 3]> {
        let mut indices = Vec::new();
        for face in &self.faces {
            for i in 1..face.len() - 1 {
                indices.push([face[0], face[i], face[i + 1]]);
            }
        }
        indices
    }

    // Primitive builders
    pub fn cube(size: f64) -> Result<Self, MeshError> {
        let half = size / 2.0;
        let positions = vec![            [-half, -half, -half], [-half, -half, half], [-half, half, -half], [-half, half, half],
            [half, -half, -half], [half, -half, half], [half, half, -half], [half, half, half],
        ];
        let faces = vec![            vec![0, 1, 3, 2], vec![4, 6, 7, 5], vec![0, 2, 6, 4],
            vec![1, 5, 7, 3], vec![0, 4, 5, 1], vec![2, 3, 7, 6],
        ];
        Self::from_faces(positions, faces)
    }

    pub fn plane(size: f64) -> Result<Self, MeshError> {
        let half = size / 2.0;
        let positions = vec![            [-half, -half, 0.0], [half, -half, 0.0], [half, half, 0.0], [-half, half, 0.0],
        ];
        let faces = vec![vec![0, 1, 2, 3]];
        Self::from_faces(positions, faces)
    }

    pub fn cylinder(radius: f64, height: f64, segments: usize) -> Result<Self, MeshError> {
        if segments < 3 {
            return Err(MeshError::NonManifold("segments must be >= 3".to_string()));
        }
        let mut positions = Vec::new();
        let mut faces = Vec::new();
        let half_height = height / 2.0;

        // Base and top vertices
        for i in 0..segments {
            let theta = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;
            positions.push([radius * theta.cos(), radius * theta.sin(), -half_height]);
            positions.push([radius * theta.cos(), radius * theta.sin(), half_height]);
        }

        // Side faces
        for i in 0..segments {
            let j = (i + 1) % segments;
            faces.push(vec![
                (2 * i) as u32,
                (2 * i + 1) as u32,
                (2 * j + 1) as u32,
                (2 * j) as u32,
            ]);
        }

        // Base and top caps
        let base_center = positions.len() as u32;
        positions.push([0.0, 0.0, -half_height]);
        let top_center = positions.len() as u32;
        positions.push([0.0, 0.0, half_height]);

        for i in 0..segments {
            let j = (i + 1) % segments;
            faces.push(vec![(2 * i) as u32, base_center, (2 * j) as u32]);
            faces.push(vec![(2 * i + 1) as u32, (2 * j + 1) as u32, top_center]);
        }

        Self::from_faces(positions, faces)
    }

    pub fn sphere(radius: f64, segments: usize, rings: usize) -> Result<Self, MeshError> {
        if segments < 3 || rings < 2 {
            return Err(MeshError::NonManifold("segments must be >= 3 and rings >= 2".to_string()));
        }
        let mut positions = Vec::new();
        let mut faces = Vec::new();
        let s = segments as u32;

        // North pole.
        let north = positions.len() as u32;
        positions.push([0.0, 0.0, radius]);

        // Intermediate rings: `rings` rows of `segments` vertices each.
        let row0 = positions.len() as u32;
        for ring in 0..rings {
            let v = (ring + 1) as f64 / (rings + 1) as f64;
            let theta = v * std::f64::consts::PI;
            for seg in 0..segments {
                let u = seg as f64 / segments as f64;
                let phi = u * 2.0 * std::f64::consts::PI;
                positions.push([
                    radius * theta.sin() * phi.cos(),
                    radius * theta.sin() * phi.sin(),
                    radius * theta.cos(),
                ]);
            }
        }

        // South pole.
        let south = positions.len() as u32;
        positions.push([0.0, 0.0, -radius]);

        let row = |j: u32| row0 + j * s;

        // North cap: triangles from the pole to the first ring.
        for seg in 0..segments {
            let next = (seg + 1) % segments;
            faces.push(vec![north, row(0) + next as u32, row(0) + seg as u32]);
        }

        // Middle: quads between adjacent rings.
        for ring in 0..rings - 1 {
            let j = ring as u32;
            for seg in 0..segments {
                let next = (seg + 1) % segments;
                let a = row(j) + seg as u32;
                let b = row(j) + next as u32;
                let d = row(j + 1) + next as u32;
                let c = row(j + 1) + seg as u32;
                faces.push(vec![a, b, d, c]);
            }
        }

        // South cap: triangles from the last ring to the south pole.
        for seg in 0..segments {
            let next = (seg + 1) % segments;
            faces.push(vec![south, row((rings - 1) as u32) + seg as u32, row((rings - 1) as u32) + next as u32]);
        }

        Self::from_faces(positions, faces)
    }

    pub fn cone(radius: f64, height: f64, segments: usize) -> Result<Self, MeshError> {
        if segments < 3 {
            return Err(MeshError::NonManifold("segments must be >= 3".to_string()));
        }
        let mut positions = Vec::new();
        let mut faces = Vec::new();

        // Apex and base vertices
        positions.push([0.0, 0.0, height]);
        for i in 0..segments {
            let theta = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;
            positions.push([radius * theta.cos(), radius * theta.sin(), 0.0]);
        }

        // Side faces
        for i in 0..segments {
            let j = (i + 1) % segments;
            faces.push(vec![0u32, (i + 1) as u32, (j + 1) as u32]);
        }

        // Base cap
        let center = positions.len() as u32;
        positions.push([0.0, 0.0, 0.0]);
        for i in 0..segments {
            let j = (i + 1) % segments;
            faces.push(vec![(i + 1) as u32, center, (j + 1) as u32]);
        }

        Self::from_faces(positions, faces)
    }

    pub fn torus(major_r: f64, minor_r: f64, major_segments: usize, minor_segments: usize) -> Result<Self, MeshError> {
        if major_segments < 3 || minor_segments < 3 {
            return Err(MeshError::NonManifold("segments must be >= 3".to_string()));
        }
        let mut positions = Vec::new();
        let mut faces = Vec::new();

        // Generate vertices
        for i in 0..major_segments {
            let u = i as f64 / major_segments as f64;
            let major_angle = u * 2.0 * std::f64::consts::PI;
            let major_x = major_r * major_angle.cos();
            let major_y = major_r * major_angle.sin();
            for j in 0..minor_segments {
                let v = j as f64 / minor_segments as f64;
                let minor_angle = v * 2.0 * std::f64::consts::PI;
                let x = major_x + minor_r * major_angle.cos() * minor_angle.cos();
                let y = major_y + minor_r * major_angle.sin() * minor_angle.cos();
                let z = minor_r * minor_angle.sin();
                positions.push([x, y, z]);
            }
        }

        // Generate faces
        for i in 0..major_segments {
            for j in 0..minor_segments {
                let next_i = (i + 1) % major_segments;
                let next_j = (j + 1) % minor_segments;
                let a = i * minor_segments + j;
                let b = i * minor_segments + next_j;
                let c = next_i * minor_segments + next_j;
                let d = next_i * minor_segments + j;
                faces.push(vec![a as u32, b as u32, c as u32, d as u32]);
            }
        }

        Self::from_faces(positions, faces)
    }

    pub fn extrude_faces(&mut self, faces: &[usize], distance: f64) -> Result<(), MeshError> {
        for &face_idx in faces {
            if face_idx >= self.faces.len() {
                return Err(MeshError::UnknownFace(face_idx));
            }
        }

        for &face_idx in faces {
            let face_verts = self.faces[face_idx].clone();
            let n = face_verts.len();

            // Calculate face normal
            let normal = self.face_normal(face_idx).ok_or_else(|| MeshError::NonManifold("degenerate face".to_string()))?;

            // Create new vertices
            for &v_idx in &face_verts {
                let v = self.positions[v_idx as usize];
                let new_v = [v[0] + normal[0] * distance, v[1] + normal[1] * distance, v[2] + normal[2] * distance];
                self.positions.push(new_v);
            }

            // Create side faces
            let base = self.positions.len() - n;
            for i in 0..n {
                let j = (i + 1) % n;
                let old_i = face_verts[i];
                let old_j = face_verts[j];
                let new_i = (base + i) as u32;
                let new_j = (base + j) as u32;
                self.faces.push(vec![old_i, old_j, new_j, new_i]);
            }

            // Update original face vertices to new positions
            self.faces[face_idx] = (0..n).map(|i| (base + i) as u32).collect();
        }

        self.is_dirty = true;
        self.rebuild_half_edges()?;
        Ok(())
    }

    pub fn inset_faces(&mut self, faces: &[usize], depth: f64) -> Result<(), MeshError> {
        let clamped_depth = depth.max(0.0).min(0.9);
        for &face_idx in faces {
            if face_idx >= self.faces.len() {
                return Err(MeshError::UnknownFace(face_idx));
            }

            let face = &self.faces[face_idx];
            let n = face.len();
            let mut centroid = [0.0; 3];

            // Calculate centroid
            for &v_idx in face {
                let v = self.positions[v_idx as usize];
                centroid[0] += v[0];
                centroid[1] += v[1];
                centroid[2] += v[2];
            }
            centroid[0] /= n as f64;
            centroid[1] /= n as f64;
            centroid[2] /= n as f64;

            // Reposition vertices
            for &v_idx in face {
                let v = &mut self.positions[v_idx as usize];
                v[0] = centroid[0] + (v[0] - centroid[0]) * (1.0 - clamped_depth);
                v[1] = centroid[1] + (v[1] - centroid[1]) * (1.0 - clamped_depth);
                v[2] = centroid[2] + (v[2] - centroid[2]) * (1.0 - clamped_depth);
            }
        }

        self.is_dirty = true;
        Ok(())
    }

    pub fn bevel_edges(&mut self, _edges: &[usize], _width: f64) -> Result<(), MeshError> {
        Err(MeshError::Unsupported("bevel"))
    }

    pub fn loop_cut(&mut self, _edges: &[usize], _cuts: u32) -> Result<(), MeshError> {
        Err(MeshError::Unsupported("loop_cut"))
    }

    pub fn face_normal(&self, face: usize) -> Option<[f64; 3]> {
        if face >= self.faces.len() || self.faces[face].len() < 3 {
            return None;
        }

        let face_vertices = &self.faces[face];
        let mut normal = [0.0; 3];

        for i in 0..face_vertices.len() {
            let a = self.positions[face_vertices[i] as usize];
            let b = self.positions[face_vertices[(i + 1) % face_vertices.len()] as usize];
            let c = self.positions[face_vertices[(i + 2) % face_vertices.len()] as usize];

            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];

            normal[0] += ab[1] * ac[2] - ab[2] * ac[1];
            normal[1] += ab[2] * ac[0] - ab[0] * ac[2];
            normal[2] += ab[0] * ac[1] - ab[1] * ac[0];
        }

        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        if len > 0.0 {
            Some([normal[0] / len, normal[1] / len, normal[2] / len])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube() {
        let mut cube = HalfEdgeMesh::cube(1.0).unwrap();
        assert_eq!(cube.vertex_count(), 8);
        assert_eq!(cube.face_count(), 6);
        assert_eq!(cube.edge_count(), 12);
        assert_eq!(cube.euler_characteristic(), 2);
        assert!(cube.is_manifold());
    }

    #[test]
    fn test_from_faces_missing_twin() {
        let positions = vec![            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
        ];
        let faces = vec![vec![0, 1, 2], vec![0, 2, 3], vec![0, 1, 2]]; // Duplicate face
        assert!(matches!(HalfEdgeMesh::from_faces(positions, faces), Err(MeshError::NonManifold(_))));
    }

    #[test]
    fn test_extrude() {
        let mut cube = HalfEdgeMesh::cube(1.0).unwrap();
        cube.extrude_faces(&[2], 0.5).unwrap();
        assert_eq!(cube.vertex_count(), 12);
        assert!(cube.is_manifold());
        assert_eq!(cube.euler_characteristic(), 2);
    }

    #[test]
    fn test_inset() {
        let mut cube = HalfEdgeMesh::cube(1.0).unwrap();
        cube.inset_faces(&[0], 0.3).unwrap();
        assert_eq!(cube.vertex_count(), 8);
        assert_eq!(cube.face_count(), 6);
        assert_eq!(cube.edge_count(), 12);
        assert!(cube.is_manifold());
        assert_eq!(cube.euler_characteristic(), 2);
    }

    #[test]
    fn test_sphere() {
        let mut sphere = HalfEdgeMesh::sphere(1.0, 8, 4).unwrap();
        assert!(sphere.is_manifold());
        assert_eq!(sphere.euler_characteristic(), 2);
    }

    #[test]
    fn test_torus() {
        let mut torus = HalfEdgeMesh::torus(1.0, 0.3, 8, 4).unwrap();
        assert!(torus.is_manifold());
        assert_eq!(torus.euler_characteristic(), 0);
    }

    #[test]
    fn test_bevel_unsupported() {
        let mut cube = HalfEdgeMesh::cube(1.0).unwrap();
        assert!(matches!(cube.bevel_edges(&[0], 0.1), Err(MeshError::Unsupported(_))));
    }

    #[test]
    fn test_triangle_indices() {
        let cube = HalfEdgeMesh::cube(1.0).unwrap();
        assert_eq!(cube.triangle_indices().len(), 12);
    }
}