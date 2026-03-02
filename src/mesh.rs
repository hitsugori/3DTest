// SPDX-License-Identifier: GPL-3.0-or-later
// 3DTest — wgpu multi-backend 3D renderer
// Copyright (C) 2026 mikedev_ <mike@mikeden.site>
//
// GPL-3.0-or-later — see COPYING or <https://www.gnu.org/licenses/>
use bytemuck::{Pod, Zeroable};
use std::f32::consts::PI;
use wgpu;
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug, Default)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal:   [f32; 3],
    pub color:    [f32; 4],
}

impl Vertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x4,
    ];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode:    wgpu::VertexStepMode::Vertex,
            attributes:   &Self::ATTRIBS,
        }
    }
}

pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices:  Vec<u32>,
}
pub fn generate_cube(face_colors: &[[f32; 4]; 6]) -> MeshData {
    let faces: [(f32, f32, f32, usize); 6] = [
        ( 0.0,  0.0,  1.0, 0), 
        ( 0.0,  0.0, -1.0, 1), 
        ( 0.0,  1.0,  0.0, 2), 
        ( 0.0, -1.0,  0.0, 3), 
        ( 1.0,  0.0,  0.0, 4), 
        (-1.0,  0.0,  0.0, 5), 
    ];
    let quad_positions: [[[f32; 3]; 4]; 6] = [
        
        [[-1.0,-1.0, 1.0],[1.0,-1.0, 1.0],[1.0, 1.0, 1.0],[-1.0, 1.0, 1.0]],
        
        [[ 1.0,-1.0,-1.0],[-1.0,-1.0,-1.0],[-1.0, 1.0,-1.0],[1.0, 1.0,-1.0]],
        
        [[-1.0, 1.0, 1.0],[1.0, 1.0, 1.0],[1.0, 1.0,-1.0],[-1.0, 1.0,-1.0]],
        
        [[-1.0,-1.0,-1.0],[1.0,-1.0,-1.0],[1.0,-1.0, 1.0],[-1.0,-1.0, 1.0]],
        
        [[ 1.0,-1.0, 1.0],[1.0,-1.0,-1.0],[1.0, 1.0,-1.0],[1.0, 1.0, 1.0]],
        
        [[-1.0,-1.0,-1.0],[-1.0,-1.0, 1.0],[-1.0, 1.0, 1.0],[-1.0, 1.0,-1.0]],
    ];
    let mut vertices = Vec::new();
    let mut indices  = Vec::new();

    for (fi, &(nx, ny, nz, ci)) in faces.iter().enumerate() {
        let base = vertices.len() as u32;
        let positions = quad_positions[fi];
        for pos in &positions {
            vertices.push(Vertex {
                position: *pos,
                normal:   [nx, ny, nz],
                color:    face_colors[ci],
            });
        }
        indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
    }
    MeshData { vertices, indices }
}



pub fn generate_sphere(color: [f32; 4], stacks: u32, slices: u32) -> MeshData {
    let mut vertices = Vec::new();
    let mut indices  = Vec::new();

    for i in 0..=stacks {
        let phi = PI * (i as f32) / (stacks as f32);
        let y   = phi.cos();
        let r   = phi.sin();

        for j in 0..=slices {
            let theta = 2.0 * PI * (j as f32) / (slices as f32);
            let x = r * theta.cos();
            let z = r * theta.sin();
            vertices.push(Vertex {
                position: [x, y, z],
                normal:   [x, y, z],
                color,
            });
        }
    }

    let cols = slices + 1;
    for i in 0..stacks {
        for j in 0..slices {
            let a = i * cols + j;
            let b = a + cols;
            indices.extend_from_slice(&[a, b, a+1, b, b+1, a+1]);
        }
    }
    MeshData { vertices, indices }
}

pub fn generate_pyramid(color: [f32; 4]) -> MeshData {
    
    let apex    = [0.0_f32, 1.2, 0.0];
    let base_y  = -0.8_f32;
    let corners = [
        [ 1.0_f32, base_y,  1.0],
        [-1.0, base_y,  1.0],
        [-1.0, base_y, -1.0],
        [ 1.0, base_y, -1.0],
    ];

    let mut vertices = Vec::new();
    let mut indices  = Vec::new();

    
    let bi = vertices.len() as u32;
    for c in &corners {
        vertices.push(Vertex { position: *c, normal: [0.0,-1.0,0.0], color });
    }
    indices.extend_from_slice(&[bi, bi+2, bi+1, bi, bi+3, bi+2]);

    
    let side_normals = [
        [ 0.0_f32, 0.447, 0.894],
        [-0.894,   0.447, 0.0  ],
        [ 0.0,     0.447,-0.894],
        [ 0.894,   0.447, 0.0  ],
    ];
    let side_pairs = [(0,1),(1,2),(2,3),(3,0)];
    for (si, &(a, b)) in side_pairs.iter().enumerate() {
        let base_idx = vertices.len() as u32;
        let n = side_normals[si];
        vertices.push(Vertex { position: apex,      normal: n, color });
        vertices.push(Vertex { position: corners[a], normal: n, color });
        vertices.push(Vertex { position: corners[b], normal: n, color });
        indices.extend_from_slice(&[base_idx, base_idx+1, base_idx+2]);
    }

    MeshData { vertices, indices }
}
pub fn generate_torus(color: [f32; 4]) -> MeshData {
    
    let major_r = 1.0_f32;
    let minor_r = 0.35_f32;
    let major_segs = 48_u32;
    let minor_segs = 12_u32;
    let ring_count = 12_u32;   

    let mut vertices = Vec::new();
    let mut indices  = Vec::new();

    
    let ring_start = vertices.len() as u32;
    for i in 0..=major_segs {
        let t = 2.0 * PI * i as f32 / major_segs as f32;
        let x = (major_r + minor_r) * t.cos();
        let z = (major_r + minor_r) * t.sin();
        let nx = t.cos();
        let nz = t.sin();
        vertices.push(Vertex { position: [x, 0.0, z], normal: [nx, 0.0, nz], color });
    }
    for i in 0..major_segs {
        indices.extend_from_slice(&[ring_start + i, ring_start + i + 1]);
    }

    
    let ring2_start = vertices.len() as u32;
    for i in 0..=major_segs {
        let t = 2.0 * PI * i as f32 / major_segs as f32;
        let x = (major_r - minor_r) * t.cos();
        let z = (major_r - minor_r) * t.sin();
        let nx = t.cos();
        let nz = t.sin();
        vertices.push(Vertex { position: [x, 0.0, z], normal: [nx, 0.0, nz], color });
    }
    for i in 0..major_segs {
        indices.extend_from_slice(&[ring2_start + i, ring2_start + i + 1]);
    }

    
    for ri in 0..ring_count {
        let phi = 2.0 * PI * ri as f32 / ring_count as f32;
        let cx  = major_r * phi.cos();
        let cz  = major_r * phi.sin();
        let ring_base = vertices.len() as u32;
        for j in 0..=minor_segs {
            let theta = 2.0 * PI * j as f32 / minor_segs as f32;
            let rad   = minor_r * theta.cos();
            let y     = minor_r * theta.sin();
            let x     = cx + rad * phi.cos();
            let z     = cz + rad * phi.sin();
            let nx    = theta.cos() * phi.cos();
            let nz    = theta.cos() * phi.sin();
            let ny    = theta.sin();
            vertices.push(Vertex { position: [x, y, z], normal: [nx, ny, nz], color });
        }
        for j in 0..minor_segs {
            indices.extend_from_slice(&[ring_base + j, ring_base + j + 1]);
        }
    }

    MeshData { vertices, indices }
}



pub fn generate_axes() -> MeshData {
    let len = 2.0_f32;
    let verts = vec![
        Vertex { position: [0.0, 0.0, 0.0], normal: [1.0,0.0,0.0], color: [1.0,0.0,0.0,1.0] },
        Vertex { position: [len, 0.0, 0.0], normal: [1.0,0.0,0.0], color: [1.0,0.0,0.0,1.0] },
        Vertex { position: [0.0, 0.0, 0.0], normal: [0.0,1.0,0.0], color: [0.0,1.0,0.0,1.0] },
        Vertex { position: [0.0, len, 0.0], normal: [0.0,1.0,0.0], color: [0.0,1.0,0.0,1.0] },
        Vertex { position: [0.0, 0.0, 0.0], normal: [0.0,0.0,1.0], color: [0.2,0.5,1.0,1.0] },
        Vertex { position: [0.0, 0.0, len], normal: [0.0,0.0,1.0], color: [0.2,0.5,1.0,1.0] },
    ];
    let idx = vec![0,1, 2,3, 4,5];
    MeshData { vertices: verts, indices: idx }
}

pub fn generate_grid(half: i32) -> MeshData {
    let mut verts = Vec::new();
    let color = [0.3_f32, 0.3, 0.3, 0.6];
    let mut idx = Vec::new();
    let mut base = 0u32;
    for i in -half..=half {
        let f = i as f32;
        let h = half as f32;
        verts.push(Vertex { position: [f,  0.0, -h], normal: [0.0,1.0,0.0], color });
        verts.push(Vertex { position: [f,  0.0,  h], normal: [0.0,1.0,0.0], color });
        verts.push(Vertex { position: [-h, 0.0,  f], normal: [0.0,1.0,0.0], color });
        verts.push(Vertex { position: [ h, 0.0,  f], normal: [0.0,1.0,0.0], color });
        idx.extend_from_slice(&[base, base+1, base+2, base+3]);
        base += 4;
    }
    MeshData { vertices: verts, indices: idx }
}

pub fn generate_normal_lines(src: &MeshData, scale: f32) -> MeshData {
    let mut verts = Vec::new();
    let mut idx   = Vec::new();
    let mut base  = 0u32;
    let color = [1.0_f32, 1.0, 0.0, 1.0];
    
    for v in src.vertices.iter().step_by(2) {
        let tip = [
            v.position[0] + v.normal[0] * scale,
            v.position[1] + v.normal[1] * scale,
            v.position[2] + v.normal[2] * scale,
        ];
        verts.push(Vertex { position: v.position, normal: v.normal, color });
        verts.push(Vertex { position: tip,         normal: v.normal, color });
        idx.extend_from_slice(&[base, base+1]);
        base += 2;
    }
    MeshData { vertices: verts, indices: idx }
}
