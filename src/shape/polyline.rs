use crate::bounding_volume::AABB;
use crate::math::{Isometry, Point, Real};
use crate::partitioning::QBVH;
use crate::shape::composite_shape::SimdCompositeShape;
use crate::shape::{FeatureId, Segment, Shape, TypedSimdCompositeShape};

#[derive(Clone)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
/// A polyline.
pub struct Polyline {
    quadtree: QBVH<u32>,
    vertices: Vec<Point<Real>>,
    indices: Vec<[u32; 2]>,
}

impl Polyline {
    /// Creates a new polyline from a vertex buffer and an index buffer.
    pub fn new(vertices: Vec<Point<Real>>, indices: Option<Vec<[u32; 2]>>) -> Self {
        let indices =
            indices.unwrap_or_else(|| (0..vertices.len() as u32 - 1).map(|i| [i, i + 1]).collect());
        let data = indices.iter().enumerate().map(|(i, idx)| {
            let aabb =
                Segment::new(vertices[idx[0] as usize], vertices[idx[1] as usize]).local_aabb();
            (i as u32, aabb)
        });

        let mut quadtree = QBVH::new();
        // NOTE: we apply no dilation factor because we won't
        // update this tree dynamically.
        quadtree.clear_and_rebuild(data, 0.0);

        Self {
            quadtree,
            vertices,
            indices,
        }
    }

    /// Compute the axis-aligned bounding box of this polyline.
    pub fn aabb(&self, pos: &Isometry<Real>) -> AABB {
        self.quadtree.root_aabb().transform_by(pos)
    }

    /// Gets the local axis-aligned bounding box of this polyline.
    pub fn local_aabb(&self) -> &AABB {
        &self.quadtree.root_aabb()
    }

    pub(crate) fn quadtree(&self) -> &QBVH<u32> {
        &self.quadtree
    }

    /// The number of segments forming this polyline.
    pub fn num_segments(&self) -> usize {
        self.indices.len()
    }

    /// An iterator through all the segments of this mesh.
    pub fn segments(&self) -> impl Iterator<Item = Segment> + '_ {
        self.indices.iter().map(move |ids| {
            Segment::new(
                self.vertices[ids[0] as usize],
                self.vertices[ids[1] as usize],
            )
        })
    }

    /// Get the `i`-th segment of this mesh.
    pub fn segment(&self, i: u32) -> Segment {
        let idx = self.indices[i as usize];
        Segment::new(
            self.vertices[idx[0] as usize],
            self.vertices[idx[1] as usize],
        )
    }

    /// Transforms  the feature-id of a segment to the feature-id of this polyline.
    pub fn segment_feature_to_polyline_feature(
        &self,
        segment: u32,
        _feature: FeatureId,
    ) -> FeatureId {
        // TODO: return a vertex feature when it makes sense.
        #[cfg(feature = "dim2")]
        return FeatureId::Face(segment);
        #[cfg(feature = "dim3")]
        return FeatureId::Edge(segment);
    }

    /// The vertex buffer of this mesh.
    pub fn vertices(&self) -> &[Point<Real>] {
        &self.vertices[..]
    }

    /// The index buffer of this mesh.
    pub fn indices(&self) -> &[[u32; 2]] {
        &self.indices
    }

    /// A flat view of the index buffer of this mesh.
    pub fn flat_indices(&self) -> &[u32] {
        unsafe {
            let len = self.indices.len() * 3;
            let data = self.indices.as_ptr() as *const u32;
            std::slice::from_raw_parts(data, len)
        }
    }
}

impl SimdCompositeShape for Polyline {
    fn map_part_at(&self, i: u32, f: &mut dyn FnMut(Option<&Isometry<Real>>, &dyn Shape)) {
        let tri = self.segment(i);
        f(None, &tri)
    }

    fn quadtree(&self) -> &QBVH<u32> {
        &self.quadtree
    }
}

impl TypedSimdCompositeShape for Polyline {
    type PartShape = Segment;
    type PartId = u32;

    #[inline(always)]
    fn map_typed_part_at(
        &self,
        i: u32,
        mut f: impl FnMut(Option<&Isometry<Real>>, &Self::PartShape),
    ) {
        let seg = self.segment(i);
        f(None, &seg)
    }

    #[inline(always)]
    fn map_untyped_part_at(&self, i: u32, mut f: impl FnMut(Option<&Isometry<Real>>, &dyn Shape)) {
        let seg = self.segment(i);
        f(None, &seg)
    }

    fn typed_quadtree(&self) -> &QBVH<u32> {
        &self.quadtree
    }
}
