// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::hardware::dsp::{
    get_biquad_coefficients, get_magnitude_response_with_precomputed, PrecomputedFreq,
};
use crate::models::Filter;
use crate::ui::state::GraphState;
use crate::ui::tokens::{
    COLOR_GRAPH_BAND_FILL, COLOR_GRAPH_BAND_STROKE, COLOR_GRAPH_GRID, COLOR_ON_SURFACE_VARIANT,
    COLOR_PRIMARY, TYPE_AXIS_LABEL,
};
use iced::widget::canvas::{Cache, Geometry, Path, Program, Stroke, Text};
use iced::{Point, Rectangle, Renderer, Theme};

type BiquadCoeffs = (f64, f64, f64, f64, f64, f64);

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

pub struct EqGraph<'a> {
    grid_cache: &'a Cache,
    curve_cache: &'a Cache,
    cached_combined_response: Vec<f64>,
    cached_band_responses: Vec<Vec<f64>>,
}

impl<'a> EqGraph<'a> {
    pub fn new(state: &'a GraphState) -> Self {
        Self {
            grid_cache: &state.grid_cache,
            curve_cache: &state.curve_cache,
            cached_combined_response: state.cached_combined_response.clone(),
            cached_band_responses: state.cached_band_responses.clone(),
        }
    }

    pub fn compute_responses(filters: &[Filter], global_gain: i8) -> (Vec<f64>, Vec<Vec<f64>>) {
        let test_freqs = &*GRAPH_FREQS;
        let coeffs: Vec<Option<BiquadCoeffs>> = filters
            .iter()
            .map(|f| {
                if f.freq == 0 || !f.enabled {
                    None
                } else {
                    Some(get_biquad_coefficients(f))
                }
            })
            .collect();

        let combined: Vec<f64> = test_freqs
            .iter()
            .map(|pf| {
                let mut total_db = global_gain as f64;
                for (b0, b1, b2, a0, a1, a2) in coeffs.iter().flatten() {
                    total_db +=
                        get_magnitude_response_with_precomputed(*b0, *b1, *b2, *a0, *a1, *a2, pf);
                }
                total_db
            })
            .collect();

        let bands: Vec<Vec<f64>> = coeffs
            .iter()
            .filter_map(|c| {
                c.map(|(b0, b1, b2, a0, a1, a2)| {
                    test_freqs
                        .iter()
                        .map(|pf| {
                            get_magnitude_response_with_precomputed(b0, b1, b2, a0, a1, a2, pf)
                        })
                        .collect()
                })
            })
            .collect();

        (combined, bands)
    }
}

lazy_static::lazy_static! {
    static ref GRAPH_FREQS: Vec<PrecomputedFreq> = {
        let points_count = 300;
        let min_f_log = 20.0f64.log10();
        let max_f_log = 20000.0f64.log10();
        let f_range_log = max_f_log - min_f_log;
        let mut freqs = Vec::with_capacity(points_count);
        for i in 0..points_count {
            let f_log = min_f_log + (i as f64 / (points_count as f64 - 1.0)) * f_range_log;
            freqs.push(PrecomputedFreq::new(10.0f64.powf(f_log)));
        }
        freqs
    };
}

impl<'a, Message> Program<Message> for EqGraph<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let grid = self.grid_cache.draw(renderer, bounds.size(), |frame| {
            let grid_color = COLOR_GRAPH_GRID;
            let text_color = COLOR_ON_SURFACE_VARIANT;

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
                    size: TYPE_AXIS_LABEL.into(),
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
                    size: TYPE_AXIS_LABEL.into(),
                    ..Default::default()
                });
            }
        });

        let curve = self.curve_cache.draw(renderer, bounds.size(), |frame| {
            let test_freqs = &*GRAPH_FREQS;
            let points_count = test_freqs.len();
            let responses = &self.cached_combined_response;
            let band_responses = &self.cached_band_responses;

            let min_db = -18.0;
            let max_db = 18.0;
            let db_range = max_db - min_db;

            for band_data in band_responses {
                let band_path = Path::new(|builder| {
                    for (i, &db) in band_data.iter().enumerate() {
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
                        .with_color(COLOR_GRAPH_BAND_STROKE)
                        .with_width(1.0),
                );
            }

            // Filled area under the combined response curve
            let zero_db_y = (1.0 - ((0.0 - min_db) / db_range)) as f32 * bounds.height;
            let fill_path = Path::new(|builder| {
                builder.move_to(Point::new(0.0, zero_db_y));
                for (i, &db) in responses.iter().enumerate() {
                    let x = (i as f32 / (points_count - 1) as f32) * bounds.width;
                    let y = (1.0 - ((db - min_db) / db_range)) as f32 * bounds.height;
                    builder.line_to(Point::new(x, y));
                }
                builder.line_to(Point::new(bounds.width, zero_db_y));
                builder.line_to(Point::new(0.0, zero_db_y));
            });
            frame.fill(&fill_path, COLOR_GRAPH_BAND_FILL);

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
                Stroke::default().with_color(COLOR_PRIMARY).with_width(3.0),
            );
        });

        vec![grid, curve]
    }
}
