use skulpin::skia_safe::*;
use skulpin::CoordinateSystemHelper;

pub struct SnowlandRenderer {
    frame_count: u64,
}

impl SnowlandRenderer {
    pub fn new() -> Self {
        SnowlandRenderer { frame_count: 0 }
    }

    pub fn draw_frame(
        &mut self,
        canvas: &mut Canvas,
        _coordinate_system_helper: CoordinateSystemHelper,
    ) {
        canvas.clear(Color4f::new(0.0, 0.0, 0.0, 0.0));

        self.frame_count += 1;

        let mut paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        paint.set_anti_alias(true);
        paint.set_style(paint::Style::Fill);

        canvas.draw_circle(Point::new(200.0, 200.0), 50.0, &paint);

        let font = Font::default();
        canvas.draw_str(
            &format!("Running for {} frames", self.frame_count),
            Point::new(400.0, 400.0),
            &font,
            &paint,
        );
    }
}
