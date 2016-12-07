use lyon_path::Path;
use lyon_path_builder::*;
use lyon_path_iterator::*;
use lyon_core::math::{Point, point};
use lyon_tessellation::geometry_builder::{VertexConstructor, VertexBuffers, BuffersBuilder};
use lyon_tessellation::path_fill::*;
use elma::lev::Level;
use Vertex;

impl VertexConstructor<Point, Vertex> for () {
    fn new_vertex(&mut self, input: Point) -> Vertex {
        Vertex {
            pos : [input.x*0.01, input.y*0.01],
            color : [0.8, 0.2, 0.3],
        }
    }
}

pub fn triangulate(level : &Level) -> VertexBuffers<Vertex> {
    // Create a simple path.
    let mut path_builder = Path::builder();
    for polygon in &level.polygons {
        if !polygon.grass {
            path_builder.move_to(point(polygon.vertices[0].x as f32, polygon.vertices[0].y as f32));

            for p in &polygon.vertices[1..] {
                path_builder.line_to(point(p.x as f32, p.y as f32));
            }
        }
    }
    path_builder.close();
    let path = path_builder.build();

    // Create the destination vertex and index buffers.
    let mut buffers: VertexBuffers<Vertex> = VertexBuffers::new();

    {
        // Create the destination vertex and index buffers.
        let mut vertex_builder = BuffersBuilder::new(&mut buffers, ()); //simple_builder(&mut buffers);

        // Create the tessellator.
        let mut tessellator = FillTessellator::new();

        // Allocate the FillEvents object and initialize it from a path iterator.
        let events = FillEvents::from_iter(path.path_iter().flattened(0.05));

        // Compute the tessellation.
        let result = tessellator.tessellate_events(
            &events,
            &FillOptions::default(),
            &mut vertex_builder
        );
        assert!(result.is_ok());
    }

    buffers
}