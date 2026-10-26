use derive_more::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub struct VertexCount(usize);

impl VertexCount {
    pub const fn new(vertices: usize) -> VertexCount {
        VertexCount(vertices)
    }
}
