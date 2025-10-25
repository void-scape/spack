use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub edge_lengths: [f32; 3],
    pub edge_luminance: [f32; 3],
    pub point_indices: [usize; 3],
}

pub fn align(
    width: usize,
    height: usize,
    points1: &[(f32, f32, f32)],
    points2: &[(f32, f32, f32)],
    threshold: f32,
) -> Vec<(Triangle, Triangle)> {
    let take = 30;
    let triangles1 = generate_all_triangles(width, height, points1, take);
    let triangles2 = generate_all_triangles(width, height, points2, take);

    let mut triangles = Vec::new();
    for t1 in triangles1.iter() {
        assert!(t1.edge_lengths.iter().all(|e| *e != 0.0));

        let ab1 = t1.edge_lengths[0] / t1.edge_lengths[1];
        let bc1 = t1.edge_lengths[1] / t1.edge_lengths[2];

        let abl1 = t1.edge_luminance[0] / t1.edge_luminance[1];
        let bcl1 = t1.edge_luminance[1] / t1.edge_luminance[2];

        for t2 in triangles2.iter() {
            assert!(t2.edge_lengths.iter().all(|e| *e != 0.0));

            let ab2 = t2.edge_lengths[0] / t2.edge_lengths[1];
            let bc2 = t2.edge_lengths[1] / t2.edge_lengths[2];

            let abl2 = t2.edge_luminance[0] / t2.edge_luminance[1];
            let bcl2 = t2.edge_luminance[1] / t2.edge_luminance[2];

            let diff = |a: f32, b: f32| (a - b).abs() < threshold;

            if diff(ab1, ab2) && diff(bc1, bc2) && diff(abl1, abl2) && diff(bcl1, bcl2) {
                triangles.push((*t1, *t2));
            }
        }
    }
    triangles
}

fn generate_all_triangles(
    width: usize,
    height: usize,
    points: &[(f32, f32, f32)],
    take: usize,
) -> Vec<Triangle> {
    let width = width as f32;
    let height = height as f32;
    let points = points.iter().take(take).copied().collect::<Vec<_>>();

    let mut triangles = Vec::with_capacity(points.len());
    let mut hash = HashSet::new();
    for i in 0..points.len() {
        for j in 0..points.len() {
            if j == i {
                continue;
            }
            for k in 0..points.len() {
                if k == j || k == i {
                    continue;
                }

                // deduplicate triangles
                let mut indices = [i, j, k];
                indices.sort();
                if !hash.insert(indices) {
                    continue;
                }

                let (p1x, p1y, p1l) = points[i];
                let (p2x, p2y, p2l) = points[j];
                let (p3x, p3y, p3l) = points[k];

                // normalize points to increase accuracy
                let p1x = p1x / width;
                let p2x = p2x / width;
                let p3x = p3x / width;
                let p1y = p1y / height;
                let p2y = p2y / height;
                let p3y = p3y / height;

                // distances
                let p1p2 = ((p1x - p2x) * (p1x - p2x) + (p1y - p2y) * (p1y - p2y)).sqrt();
                let p2p3 = ((p2x - p3x) * (p2x - p3x) + (p2y - p3y) * (p2y - p3y)).sqrt();
                let p1p3 = ((p1x - p3x) * (p1x - p3x) + (p1y - p3y) * (p1y - p3y)).sqrt();

                // sort edges
                let mut edge_lengths = [p1p2, p2p3, p1p3];
                edge_lengths.sort_by(|a, b| a.total_cmp(b));
                let mut edge_luminance = [p1l, p2l, p3l];
                edge_luminance.sort_by(|a, b| a.total_cmp(b));
                let point_indices = [i, j, k];

                triangles.push(Triangle {
                    edge_lengths,
                    edge_luminance,
                    point_indices,
                })
            }
        }
    }
    triangles
}
