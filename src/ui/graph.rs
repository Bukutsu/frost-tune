use iced::widget::canvas::{Program, Geometry, Path, Stroke, Frame, Text};
use iced::{Color, Rectangle, Renderer, Theme, Point};
use crate::models::Filter;
use crate::hardware::dsp::{calculate_total_response, get_magnitude_response};

use crate::ui::theme::TOKYO_NIGHT_PRIMARY;

pub struct EqGraph {
    filters: Vec<Filter>,
    global_gain: i8,
}

impl EqGraph {
    pub fn new(filters: &[Filter], global_gain: i8) -> Self {
        Self {
            filters: filters.to_vec(),
            global_gain,
        }
    }
}

impl<Message> Program<Message> for EqGraph {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let palette = theme.palette();

        // 1. Draw Grid
        let grid_color = Color::from_rgba(0.5, 0.5, 0.5, 0.2);
        let text_color = palette.text;

        // Vertical lines (Frequency)
        let freqs: [f64; 10] = [20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0];
        let min_f_log = 20.0f64.log10();
        let max_f_log = 20000.0f64.log10();
        let f_range_log = max_f_log - min_f_log;

        for &f in &freqs {
            let x = (f.log10() - min_f_log) / f_range_log * bounds.width as f64;
            let path = Path::line(Point::new(x as f32, 0.0), Point::new(x as f32, bounds.height));
            frame.stroke(&path, Stroke::default().with_color(grid_color).with_width(1.0));

            let label = if f >= 1000.0 {
                format!("{}k", f / 1000.0)
            } else {
                format!("{}", f)
            };
            frame.fill_text(Text {
                content: label,
                position: Point::new(x as f32 + 2.0, bounds.height - 15.0),
                color: text_color,
                size: 10.0.into(),
                ..Default::default()
            });
        }

        // Horizontal lines (dB)
        let dbs = [-15.0, -10.0, -5.0, 0.0, 5.0, 10.0, 15.0];
        let min_db = -18.0;
        let max_db = 18.0;
        let db_range = max_db - min_db;

        for &db in &dbs {
            let y = (1.0 - (db - min_db) / db_range) * bounds.height as f64;
            let path = Path::line(Point::new(0.0, y as f32), Point::new(bounds.width, y as f32));
            frame.stroke(&path, Stroke::default().with_color(grid_color).with_width(1.0));

            frame.fill_text(Text {
                content: format!("{}{}dB", if db > 0.0 { "+" } else { "" }, db),
                position: Point::new(5.0, y as f32 - 12.0),
                color: text_color,
                size: 10.0.into(),
                ..Default::default()
            });
        }

        // 2. Draw Curve
        let points_count = 300;
        let mut test_freqs = Vec::with_capacity(points_count);
        for i in 0..points_count {
            let f_log = min_f_log + (i as f64 / (points_count as f64 - 1.0)) * f_range_log;
            test_freqs.push(10.0f64.powf(f_log));
        }

        let responses = calculate_total_response(&self.filters, self.global_gain, &test_freqs);

        // Draw faint individual enabled bands first
        for filter in self.filters.iter().filter(|f| f.enabled) {
            let band_path = Path::new(|builder| {
                for (i, &f) in test_freqs.iter().enumerate() {
                    let db = get_magnitude_response(filter, f);
                    let x = (i as f32 / (points_count - 1) as f32) * bounds.width;
                    let y = (1.0 - ((db - min_db) / db_range)) as f32 * bounds.height;
                    let p = Point::new(x, y);
                    if i == 0 {
                        builder.move_to(p);
                    } else {
                        builder.line_to(p);
                    }
                }
            });

            frame.stroke(
                &band_path,
                Stroke::default()
                    .with_color(Color::from_rgba(0.49, 0.81, 1.0, 0.25))
                    .with_width(1.0),
            );
        }

        let path = Path::new(|builder| {
            for (i, &db) in responses.iter().enumerate() {
                let x = (i as f32 / (points_count - 1) as f32) * bounds.width;
                let y = (1.0 - ((db - min_db) / db_range)) as f32 * bounds.height;
                let p = Point::new(x, y);
                if i == 0 {
                    builder.move_to(p);
                } else {
                    builder.line_to(p);
                }
            }
        });

        frame.stroke(&path, Stroke::default().with_color(TOKYO_NIGHT_PRIMARY).with_width(2.0));

        vec![frame.into_geometry()]
    }
}
