use cgmath::{vec2, InnerSpace, Matrix2, One, Rad, SquareMatrix, Vector2};

#[derive(Copy, Clone)]
pub struct Transform {
    position: Vector2<f64>,
    matrix: Matrix2<f64>,
}

impl Transform {
    pub fn transform(&self, p: Vector2<f64>) -> Vector2<f64> {
        self.position + self.matrix * p
    }

    pub fn unit() -> Self {
        Transform {
            position: vec2(0.0, 0.0),
            matrix: Matrix2::one(),
        }
    }

    pub fn inverse(&self) -> Self {
        let matrix = self.matrix.invert().unwrap();

        Transform {
            position: -matrix * self.position,
            matrix,
        }
    }

    pub fn translate(&self, to: Vector2<f64>) -> Self {
        Transform {
            position: self.position + self.matrix * to,
            matrix: self.matrix,
        }
    }

    pub fn rotate(&self, angle: f64) -> Self {
        Transform {
            position: self.position,
            matrix: self.matrix * Matrix2::from_angle(Rad(angle)),
        }
    }

    pub fn scale(&self, scale: f64) -> Self {
        Transform {
            position: self.position,
            matrix: self.matrix * scale,
        }
    }

    pub fn scale2(&self, scale: Vector2<f64>) -> Self {
        Transform {
            position: self.position,
            matrix: self.matrix * Matrix2::new(scale.x, 0.0, 0.0, scale.y),
        }
    }

    // p1–p2: line to draw image along
    // bx: length of image used before (x1, y1)
    // br: length of image used after (x2, y2)
    // by: proportional (of ih) y offset within the image the line is conceptually along
    // ih: image height
    pub fn skew(
        &self,
        bx: f64,
        by: f64,
        br: f64,
        ih: f64,
        p1: Vector2<f64>,
        p2: Vector2<f64>,
    ) -> Self {
        let v = p2 - p1;

        self.translate(p1)
            .rotate(v.y.atan2(v.x))
            .translate(vec2(-bx, by * ih))
            .scale2(vec2(bx + br + v.magnitude(), ih))
    }
}
