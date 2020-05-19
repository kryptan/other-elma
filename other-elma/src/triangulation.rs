use crate::render::Vertex;
use elma::lev::Level;
use lyon_tessellation::geom::math::{point, Point};
use lyon_tessellation::geometry_builder::{BuffersBuilder, VertexBuffers};
use lyon_tessellation::path::Path;
use lyon_tessellation::{FillAttributes, FillOptions, FillTessellator, FillVertexConstructor};

impl FillVertexConstructor<Vertex> for () {
    fn new_vertex(&mut self, input: Point, _attributes: FillAttributes) -> Vertex {
        Vertex {
            position: [input.x, input.y],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coord: [0.0, 0.0],
            tex_bounds: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

pub fn triangulate(level: &Level) -> VertexBuffers<Vertex, u32> {
    // Create a simple path.
    let mut path_builder = Path::builder();
    for polygon in &level.polygons {
        if !polygon.grass {
            path_builder.move_to(point(
                polygon.vertices[0].x as f32,
                polygon.vertices[0].y as f32,
            ));

            for p in &polygon.vertices[1..] {
                path_builder.line_to(point(p.x as f32, p.y as f32));
            }
        }
    }
    path_builder.close();
    let path = path_builder.build();

    // Create the destination vertex and index buffers.
    let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();

    // Create the destination vertex and index buffers.
    let mut vertex_builder = BuffersBuilder::new(&mut buffers, ()); //simple_builder(&mut buffers);

    // Create the tessellator.
    let mut tessellator = FillTessellator::new();

    // Compute the tessellation.
    let result = tessellator.tessellate_path(&path, &FillOptions::default(), &mut vertex_builder);
    assert!(result.is_ok());

    buffers
}
