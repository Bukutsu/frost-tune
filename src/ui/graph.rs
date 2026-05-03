use crate::hardware::dsp::{calculate_total_response, get_biquad_coefficients, get_magnitude_response_with_coeffs};
use crate::models::Filter;
use iced::widget::canvas::{Cache, Geometry, Path, Program, Stroke, Text};
use iced::{Color, Point, Rectangle, Renderer, Theme};

use crate::ui::theme::{TOKYO_NIGHT_FG_DARK, TOKYO_NIGHT_PRIMARY};

#[derive(Debug, Clone, Copy)]
pub struct GraphLabelLayout {
    pub title_pos: Point,
    pub freq_axis_pos: Point,
    pub gain_axis_pos: Point,
    pub legend_combined_pos: Point,
}

pub fn graph_label_layout(width: f32, height: f32) -> GraphLabelLayout {
    let safe_width = width.max(320.0);
    let safe_height = height.max(180.0);

    GraphLabelLayout {
        title_pos: Point::new(safe_width / 2.0, 24.0),
        freq_axis_pos: Point::new(safe_width / 2.0, safe_height - 16.0),
        gain_axis_pos: Point::new(24.0, safe_height / 2.0),
        legend_combined_pos: Point::new((safe_width - 120.0).max(240.0), 24.0),
    }
}

pub struct EqGraph {
    filters: Vec<Filter>,
    global_gain: i8,
    grid_cache: Cache,
    curve_cache: Cache,
}

impl EqGraph {
    pub fn new(filters: &[Filter], global_gain: i8) -> Self {
        let graph = Self {
            filters: filters.to_vec(),
            global_gain,
            grid_cache: Cache::new(),
            curve_cache: Cache::new(),
        };
        graph.curve_cache.clear();
        graph
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
        let grid = self.grid_cache.draw(renderer, bounds.size(), |frame| {
            let _palette = theme.palette();
            let grid_color = Color::from_rgba(0.5, 0.5, 0.5, 0.2);
            let text_color = TOKYO_NIGHT_FG_DARK;

            let freqs: [f64; 10] = [
                20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0,
            ];
            let min_f_log = 20.0f64.log10();
            let max_f_log = 20000.0f64.log10();
            let f_range_log = max_f_log - min_f_log;

            for &f in &freqs {
                let x = (f.log10() - min_f_log) / f_range_log * bounds.width as f64;
                let path = Path::line(
                    Point::new(x as f32, 0.0),
                    Point::new(x as f32, bounds.height),
                );
                frame.stroke(
                    &path,
                    Stroke::default().with_color(grid_color).with_width(1.0),
                );

                let label = if f >= 1000.0 {
                    format!("{}k", f / 1000.0)
                } else {
                    format!("{}", f)
                };
                frame.fill_text(Text {
                    content: label,
                    position: Point::new(x as f32 + 2.0, bounds.height - 15.0),
                    color: text_color,
                    size: 12.0.into(),
                    ..Default::default()
                });
            }

            let dbs = [-15.0, -10.0, -5.0, 0.0, 5.0, 10.0, 15.0];
            let min_db = -18.0;
            let max_db = 18.0;
            let db_range = max_db - min_db;

            for &db in &dbs {
                let y = (1.0 - (db - min_db) / db_range) * bounds.height as f64;
                let path = Path::line(
                    Point::new(0.0, y as f32),
                    Point::new(bounds.width, y as f32),
                );
                frame.stroke(
                    &path,
                    Stroke::default().with_color(grid_color).with_width(1.0),
                );

                frame.fill_text(Text {
                    content: format!("{}{}dB", if db > 0.0 { "+" } else { "" }, db),
                    position: Point::new(5.0, y as f32 - 12.0),
                    color: text_color,
                    size: 12.0.into(),
                    ..Default::default()
                });
            }
        });

        let curve = self.curve_cache.draw(renderer, bounds.size(), |frame| {
            let points_count = 300;
            let min_f_log = 20.0f64.log10();
            let max_f_log = 20000.0f64.log10();
            let f_range_log = max_f_log - min_f_log;
            let mut test_freqs = Vec::with_capacity(points_count);
            for i in 0..points_count {
                let f_log = min_f_log + (i as f64 / (points_count as f64 - 1.0)) * f_range_log;
                test_freqs.push(10.0f64.powf(f_log));
            }

            let responses = calculate_total_response(&self.filters, self.global_gain, &test_freqs);

            let min_db = -18.0;
            let max_db = 18.0;
            let db_range = max_db - min_db;

            for filter in &self.filters {
                if filter.freq == 0 || !filter.enabled {
                    continue;
                }
                let (b0, b1, b2, a0, a1, a2) = get_biquad_coefficients(filter);
                let band_path = Path::new(|builder| {
                    for (i, &f) in test_freqs.iter().enumerate() {
                        let db = get_magnitude_response_with_coeffs(b0, b1, b2, a0, a1, a2, f);
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

            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(TOKYO_NIGHT_PRIMARY)
                    .with_width(2.0),
            );
        });

        vec![grid, curve]
    }
}
