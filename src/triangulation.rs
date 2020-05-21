use elma::lev::Level;
use lyon_tessellation::geom::math::{point, Point};
use lyon_tessellation::geometry_builder::{BuffersBuilder, VertexBuffers};
use lyon_tessellation::path::Path;
use lyon_tessellation::{FillAttributes, FillOptions, FillTessellator};

pub fn triangulate<V>(
    level: &Level,
    grass: bool,
    f: impl Fn([f32; 2]) -> V,
) -> VertexBuffers<V, u32> {
    let mut path_builder = Path::builder();
    for polygon in &level.polygons {
        if polygon.grass == grass {
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
    let mut buffers: VertexBuffers<V, u32> = VertexBuffers::new();

    // Create the destination vertex and index buffers.
    let mut vertex_builder =
        BuffersBuilder::new(&mut buffers, |input: Point, _attributes: FillAttributes| {
            f([input.x, input.y])
        });

    let result =
        FillTessellator::new().tessellate_path(&path, &FillOptions::default(), &mut vertex_builder);
    assert!(result.is_ok());

    buffers
}
