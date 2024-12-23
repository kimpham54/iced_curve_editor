// IMPLEMENTS ALGORITHM MANUALLY
use iced::widget::canvas::{Path, Stroke};
use iced::widget::{button, canvas, container, horizontal_space, hover, Canvas};
use iced::{mouse, Element, Fill, Point, Rectangle, Theme};
use splines::{Interpolation, Key, Spline};

pub fn main() -> iced::Result {
    // Entry point of the application. This initializes and runs the application.
    // `iced::application` creates an application instance, passing the update and view methods.
    iced::application(
        "Point and Curve Editor Tool - A Rust Iced application",
        ExampleCanvas::update,
        ExampleCanvas::view,
    )
    .theme(|_| Theme::CatppuccinMocha)
    .run() // Start the application.
}

#[derive(Default)]
struct ExampleCanvas {
    dotstate: DotState,
    dots: Vec<Dot>,
    straight_mode: bool,
    curve_mode: Option<CurveAlgorithm>,
    delete_mode: bool,
}

#[derive(Debug, Clone, Copy)]
enum CurveAlgorithm {
    CatmullRom,
    MonotonicSpline,
    NaturalCubicSpline,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    AddDot(Dot), // Message to add a new point.
    Clear,       // Message to clear all points.
    Straight,    // Toggle straight line connector mode on and off.
    Curve, // Toggle curve line connector mode between catmull rom splines and off, can add more in future if needed
    DeleteMode,
    DeleteDot(Point), //not Dot for easier proximity matching
}

impl ExampleCanvas {
    fn update(&mut self, message: Message) {
        match message {
            Message::AddDot(dot) => {
                self.dots.push(dot);
                self.dotstate.request_redraw();
                if self.straight_mode {
                    self.dotstate.request_redraw(); // Ensure lines are regenerated
                }
            }
            Message::Clear => {
                self.dotstate = DotState::default();
                self.dots.clear();
                self.straight_mode = false; // Reset the mode
                self.curve_mode = None;
            }
            Message::Straight => {
                self.straight_mode = !self.straight_mode; // Toggle the mode
                self.dotstate.request_redraw(); // Redraw to show/hide lines
            }
            Message::Curve => {
                // Cycle through curve modes: Off -> Catmull-Rom -> Off
                self.curve_mode = match self.curve_mode {
                    None => Some(CurveAlgorithm::CatmullRom), // 1st press
                    Some(CurveAlgorithm::CatmullRom) => Some(CurveAlgorithm::MonotonicSpline),
                    Some(CurveAlgorithm::MonotonicSpline) => Some(CurveAlgorithm::NaturalCubicSpline),
                    Some(CurveAlgorithm::NaturalCubicSpline) => None, // 2nd press

                                                             // Some(CurveAlgorithm::CatmullRom) => None, // 2nd press
                };
                self.dotstate.request_redraw();
            }
            Message::DeleteDot(position) => {
                if self.delete_mode {
                    // Find the nearest dot and remove it
                    if let Some(index) = self
                        .dots
                        .iter()
                        .position(|dot| (dot.position.x - position.x).abs() < 10.0 && (dot.position.y - position.y).abs() < 10.0)
                    {
                        self.dots.remove(index);
                        self.dotstate.request_redraw();
                    }
                }
            }
            
            Message::DeleteMode => {
                self.delete_mode = !self.delete_mode; // Toggle delete mode
            }
        }
    }

    /// Builds the user interface (UI) for the application.
    fn view(&self) -> Element<Message> {
        container(hover(
            self.dotstate
                .view(&self.dots, self.straight_mode, self.curve_mode, self.delete_mode)
                .map(|dot| {
                    if self.delete_mode {
                        Message::DeleteDot(dot.position)
                    } else {
                        Message::AddDot(dot)
                    }
                }),
            if self.dots.is_empty() {
                container(horizontal_space())
            } else {
                container(
                    iced::widget::column![
                        button("Clear")
                            .style(button::danger)
                            .on_press(Message::Clear),
                        button(if self.straight_mode {
                            "Straight: On"
                        } else {
                            "Straight: Off"
                        })
                        .on_press(Message::Straight),
                        if self.dots.len() >= 2 {
                            // Only enable curve button if there are >2 points
                            button(match self.curve_mode {
                                None => "Curve: Off",
                                Some(CurveAlgorithm::CatmullRom) => "Curve: Catmull-Rom",
                                Some(CurveAlgorithm::MonotonicSpline) => "Curve: Monotonic",
                                Some(CurveAlgorithm::NaturalCubicSpline) => "Curve: Natural Cubic",
                            })
                            .on_press(Message::Curve) // Button is active
                        } else {
                            button("Curve: Disabled") // Button is disabled (no `on_press`)
                        },
                        button(if self.delete_mode {
                            "Delete Mode: On"
                        } else {
                            "Delete Mode: Off"
                        })
                        .on_press(Message::DeleteMode),
                    ]
                    .spacing(10),
                )
                .padding(10)
                .align_right(Fill)
            },
        ))
        .padding(20)
        .into()
    }
}

#[derive(Debug, Clone, Copy)]
struct Dot {
    position: iced::Point,
}

// impl Dot{
// fn draw_all(curves: &[Curve], frame: &mut Frame, theme: &Theme) {
// }
// }

#[derive(Default)]
struct DotState {
    cache: canvas::Cache,
}

impl DotState {
    pub fn view<'a>(
        &'a self,
        dots: &'a [Dot],
        straight_mode: bool,
        curve_mode: Option<CurveAlgorithm>,
        delete_mode: bool,
    ) -> Element<'a, Dot> {
        Canvas::new(DrawDotsAndLines {
            state: self,
            dots,
            straight_mode,
            curve_mode,
            delete_mode
        }) //Pass straight_mode to DrawDotsandLines
        .width(Fill)
        .height(Fill)
        .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear();
    }
}
struct DrawDotsAndLines<'a> {
    state: &'a DotState,
    dots: &'a [Dot],
    straight_mode: bool,
    curve_mode: Option<CurveAlgorithm>,
    delete_mode: bool,
}

impl canvas::Program<Dot> for DrawDotsAndLines<'_> {
    type State = DotState;

    /// Handles events on the canvas, such as mouse clicks.
    fn update(
        &self,
        _state: &mut Self::State,
        event: iced::widget::canvas::event::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (iced::widget::canvas::event::Status, Option<Dot>) {
        // Only handle events when the cursor is inside the canvas.
        let Some(cursor_position) = cursor.position_in(bounds) else {
            return (iced::widget::canvas::event::Status::Ignored, None);
        };

        // Handle left mouse button presses to add a new dot.
    //     match event {
    //         iced::widget::canvas::event::Event::Mouse(iced::mouse::Event::ButtonPressed(
    //             iced::mouse::Button::Left,
    //         )) => {
    //             let dot = Dot {
    //                 position: cursor_position,
    //             };
    //             (iced::widget::canvas::event::Status::Captured, Some(dot))
    //         }
    //         _ => (iced::widget::canvas::event::Status::Ignored, None),
    //     }
    // }

    match event {
        iced::widget::canvas::event::Event::Mouse(iced::mouse::Event::ButtonPressed(
            iced::mouse::Button::Left,
        )) => {
            if self.delete_mode {
                // Directly return the position for deletion
                return (
                    iced::widget::canvas::event::Status::Captured,
                    Some(Dot {
                        position: cursor_position,
                    }),
                );
            } else {
                // Handle adding a new dot
                let dot = Dot {
                    position: cursor_position,
                };
                return (
                    iced::widget::canvas::event::Status::Captured,
                    Some(dot),
                );
            }
        }
        _ => (iced::widget::canvas::event::Status::Ignored, None),
    }
}



    /// Draws the canvas content.
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let content = self.state.cache.draw(renderer, bounds.size(), |frame| {
            // Draw grid lines
            let grid_spacing = 50.0; // Spacing between grid lines in pixels
            let bounds_width = bounds.width;
            let bounds_height = bounds.height;

            // Draw vertical grid lines
            for x in (0..bounds_width as usize).step_by(grid_spacing as usize) {
                let x = x as f32;
                frame.stroke(
                    &Path::line(
                        Point { x, y: 0.0 },
                        Point {
                            x,
                            y: bounds_height,
                        },
                    ),
                    Stroke::default()
                        .with_width(1.0)
                        .with_color(iced::Color::from_rgb(0.23, 0.25, 0.18)), // Light gray
                );
            }

            // Draw horizontal grid lines
            for y in (0..bounds_height as usize).step_by(grid_spacing as usize) {
                let y = y as f32;
                frame.stroke(
                    &Path::line(Point { x: 0.0, y }, Point { x: bounds_width, y }),
                    Stroke::default()
                        .with_width(1.0)
                        .with_color(iced::Color::from_rgb(0.21, 0.23, 0.19)), // Light gray
                );
            }

            // Draw border
            frame.stroke(
                &Path::rectangle(Point::ORIGIN, frame.size()),
                Stroke::default()
                    .with_width(4.0)
                    .with_color(iced::Color::from_rgb(0.23, 0.37, 0.80)),
            );
            // Draw dots - iterate list and draw on the canvas.
            for dot in self.dots {
                // Use the x and y fields of the iced::Point to draw a circle at dot position.
                frame.fill(
                    &iced::widget::canvas::Path::circle(dot.position, 5.0), // Use *position to dereference the Point reference.
                    iced::Color {
                        r: 0.44, // 111 / 255
                        g: 0.20, // 50 / 255
                        b: 0.0,  // 0 / 255
                        a: 1.0,  // Fully opaque
                    }, // Use the theme's text color for the dot.
                );
            }

            let mut sorted_dots: Vec<Dot> = Vec::new(); // Kim: Outside of if statement so it can be used below
            dbg!(&sorted_dots);

            if !self.dots.is_empty() {
                if self.straight_mode || self.curve_mode.is_some() {
                    sorted_dots = self.dots.to_vec();
                    sorted_dots.sort_by(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap());
                    let first_dot = sorted_dots.first().unwrap();
                    let last_dot = sorted_dots.last().unwrap();
                    let first_control_dot = Dot {
                        position: Point {
                            x: 0.0,
                            y: first_dot.position.y,
                        },
                    };
                    let last_control_dot = Dot {
                        position: Point {
                            x: bounds_width,
                            y: last_dot.position.y,
                        },
                    };
                    sorted_dots.push(first_control_dot);
                    sorted_dots.push(last_control_dot);
                    sorted_dots.sort_by(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap());
                    dbg!(&sorted_dots);

                    // Prepare knots and knot spacing
                    // let first_dot = sorted_dots.first().unwrap();
                    // let last_dot = sorted_dots.last().unwrap();
                    // let mut x_knots: Vec<f32> = vec![0.0, last_dot.position.x]; // BUG: knots break if less than 4
                    // let mut y_knots: Vec<f32> = vec![first_dot.position.y, last_dot.position.y];
                    // x_knots.extend(sorted_dots.iter().map(|dot| dot.position.x));
                    // y_knots.extend(sorted_dots.iter().map(|dot| dot.position.y));
                    // x_knots.push(1.0);
                    // y_knots.push(last_dot.position.y);
                }
            }

            // Draw straight line connectors if "Straight" is active
            if self.straight_mode {
                for i in 0..sorted_dots.len() - 1 {
                    let start = sorted_dots[i].position;
                    let end = sorted_dots[i + 1].position;

                    frame.stroke(
                        &iced::widget::canvas::Path::line(start, end),
                        iced::widget::canvas::Stroke::default()
                            .with_width(2.0)
                            .with_color(theme.palette().text),
                    );
                }
            }

            // Handle curve drawing
            if let Some(curve_mode) = self.curve_mode {
                let mut path = Path::new(|builder| {
                    match curve_mode {
                        CurveAlgorithm::CatmullRom => {
                            let n_points = 200; // Number of interpolated points
                            for i in 0..n_points {
                                let t = i as f32 / (n_points - 1) as f32;
                                let segment_index =
                                    ((sorted_dots.len() - 1) as f32 * t).floor() as usize;

                                // Clamp control points for safe interpolation
                                let p0 = if segment_index == 0 {
                                    &sorted_dots[segment_index]
                                } else {
                                    &sorted_dots[segment_index - 1]
                                };
                                let p1 = &sorted_dots[segment_index];
                                let p2 = &sorted_dots
                                    [std::cmp::min(segment_index + 1, sorted_dots.len() - 1)];
                                let p3 = &sorted_dots
                                    [std::cmp::min(segment_index + 2, sorted_dots.len() - 1)];

                                let local_t = (t * (sorted_dots.len() - 1) as f32) % 1.0;

                                let alpha = 0.3; // Adjust as needed, but 0.5 is typical for centripetal splines

                                let (x, y) = (
                                    catmull_rom_centripetal(
                                        local_t,
                                        p0.position.x,
                                        p1.position.x,
                                        p2.position.x,
                                        p3.position.x,
                                        alpha,
                                    ),
                                    catmull_rom_centripetal(
                                        local_t,
                                        p0.position.y,
                                        p1.position.y,
                                        p2.position.y,
                                        p3.position.y,
                                        alpha,
                                    ),
                                );

                                if i == 0 {
                                    builder.move_to(Point { x, y });
                                } else {
                                    builder.line_to(Point { x, y });
                                }
                            }
                        }
                        CurveAlgorithm::MonotonicSpline => {
                            let xs: Vec<f32> =
                                sorted_dots.iter().map(|dot| dot.position.x).collect();
                            let ys: Vec<f32> =
                                sorted_dots.iter().map(|dot| dot.position.y).collect();

                            if xs.len() < 2 {
                                // Skip drawing if not enough points
                                return;
                            }

                            if let Some(interpolated_points) = monotonic_cubic_spline(&xs, &ys) {
                                for (i, &(x, y)) in interpolated_points.iter().enumerate() {
                                    // Ensure finite values for rendering
                                    if !x.is_finite() || !y.is_finite() {
                                        eprintln!("Invalid interpolated point: x={}, y={}", x, y);
                                        continue; // Skip invalid points
                                    }

                                    if i == 0 {
                                        builder.move_to(Point { x, y });
                                    } else {
                                        builder.line_to(Point { x, y });
                                    }
                                }
                            } else {
                                eprintln!("Monotonic cubic spline interpolation failed.");
                            }
                        }

                        CurveAlgorithm::NaturalCubicSpline => {
                            if sorted_dots.len() >= 2 {
                                let mut x_points: Vec<f32> =
                                    sorted_dots.iter().map(|dot| dot.position.x).collect();
                                let mut y_points: Vec<f32> =
                                    sorted_dots.iter().map(|dot| dot.position.y).collect();

                                // Compute natural cubic spline coefficients
                                let x_spline = compute_natural_cubic_spline(&x_points);
                                let y_spline = compute_natural_cubic_spline(&y_points);

                                let n_points_per_segment = 50; // Number of points per segment for smoothness
                                for i in 0..sorted_dots.len() - 1 {
                                    for j in 0..n_points_per_segment {
                                        let t = j as f32 / (n_points_per_segment as f32);

                                        let x = evaluate_cubic(&x_spline[i], t);
                                        let y = evaluate_cubic(&y_spline[i], t);

                                        if i == 0 && j == 0 {
                                            builder.move_to(Point { x, y });
                                        } else {
                                            builder.line_to(Point { x, y });
                                        }
                                    }
                                }
                            }
                        }
                    }
                });

                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_width(2.0)
                        .with_color(theme.palette().text),
                );
            }
        });

        vec![content]
    }
}

fn safe_powf_distance(a: f32, b: f32, alpha: f32) -> f32 {
    let dist = (a - b).abs();
    if dist < 1e-9 {
        // fallback to small epsilon
        1e-9_f32.powf(alpha)
    } else {
        dist.powf(alpha)
    }
}

fn catmull_rom_centripetal(t: f32, p0: f32, p1: f32, p2: f32, p3: f32, alpha: f32) -> f32 {
    let d01 = safe_powf_distance(p0, p1, alpha);
    let d12 = safe_powf_distance(p1, p2, alpha);
    let d23 = safe_powf_distance(p2, p3, alpha);

    let t0 = 0.0;
    let t1 = t0 + d01;
    let t2 = t1 + d12;
    let t3 = t2 + d23;

    // We only interpolate between p1 and p2, so we re-map t from [0..1] to [t1..t2].
    let t = t1 + (t * (t2 - t1));

    // If t1==t0, t2==t1, or t3==t2 (which can happen if d01/d12/d23 = 0), we can early-return:
    if (t1 - t0).abs() < 1e-12 || (t2 - t1).abs() < 1e-12 || (t3 - t2).abs() < 1e-12 {
        // fallback: linear interpolation or just pick p1
        return p1;
    }

    // ... compute A1, A2, A3
    let a1 = (t1 - t) / (t1 - t0) * p0 + (t - t0) / (t1 - t0) * p1;
    let a2 = (t2 - t) / (t2 - t1) * p1 + (t - t1) / (t2 - t1) * p2;
    let a3 = (t3 - t) / (t3 - t2) * p2 + (t - t2) / (t3 - t2) * p3;

    // ... then B1, B2, final
    let b1 = (t2 - t) / (t2 - t0) * a1 + (t - t0) / (t2 - t0) * a2;
    let b2 = (t3 - t) / (t3 - t1) * a2 + (t - t1) / (t3 - t1) * a3;

    (t2 - t) / (t2 - t1) * b1 + (t - t1) / (t2 - t1) * b2
}

fn monotonic_cubic_spline(xs: &[f32], ys: &[f32]) -> Option<Vec<(f32, f32)>> {
    if xs.len() != ys.len() || xs.len() < 2 {
        return None; // Invalid input
    }

    let n = xs.len();
    let mut slopes = vec![0.0; n - 1];
    let mut tangents = vec![0.0; n];

    // Step 1: Compute slopes between points
    for i in 0..n - 1 {
        slopes[i] = (ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i]);
    }

    // Step 2: Compute tangents
    tangents[0] = slopes[0];
    tangents[n - 1] = slopes[n - 2];
    for i in 1..n - 1 {
        tangents[i] = (slopes[i - 1] + slopes[i]) / 2.0;
    }

    // Step 3: Adjust tangents to ensure monotonicity
    for i in 0..n - 1 {
        if slopes[i] == 0.0 {
            tangents[i] = 0.0;
            tangents[i + 1] = 0.0;
        } else {
            let alpha = tangents[i] / slopes[i];
            let beta = tangents[i + 1] / slopes[i];
            let s = alpha * alpha + beta * beta;
            if s > 9.0 {
                let tau = 3.0 / s.sqrt();
                tangents[i] = tau * alpha * slopes[i];
                tangents[i + 1] = tau * beta * slopes[i];
            }
        }
    }

    // Step 4: Interpolate
    let mut result = Vec::new();
    let num_points = 100; // Number of points to sample for rendering
    for i in 0..n - 1 {
        let x0 = xs[i];
        let x1 = xs[i + 1];
        let y0 = ys[i];
        let y1 = ys[i + 1];
        let t0 = tangents[i];
        let t1 = tangents[i + 1];

        for j in 0..=num_points {
            let t = j as f32 / num_points as f32;
            let h00 = (1.0 + 2.0 * t) * (1.0 - t) * (1.0 - t);
            let h10 = t * (1.0 - t) * (1.0 - t);
            let h01 = t * t * (3.0 - 2.0 * t);
            let h11 = t * t * (t - 1.0);

            let x = x0 + t * (x1 - x0);
            let y = h00 * y0 + h10 * (x1 - x0) * t0 + h01 * y1 + h11 * (x1 - x0) * t1;

            result.push((x, y));
        }
    }

    Some(result)
}
fn quadratic_bezier(t: f32, p0: f32, p1: f32, p2: f32) -> f32 {
    let u = 1.0 - t;
    (u * u * p0) + (2.0 * u * t * p1) + (t * t * p2)
}

fn compute_natural_cubic_spline(points: &[f32]) -> Vec<[f32; 4]> {
    let n = points.len() - 1;
    let mut a = points.to_vec();
    let mut b = vec![0.0; n];
    let mut d = vec![0.0; n];
    let mut h = vec![0.0; n];
    let mut alpha = vec![0.0; n];

    for i in 0..n {
        // h[i] = 1.0; // Assuming equal spacing between points
        h[i] = 1.0;

    }

    for i in 1..n {
        alpha[i] = (3.0 / h[i]) * (a[i + 1] - a[i]) - (3.0 / h[i - 1]) * (a[i] - a[i - 1]);
    }

    let mut c = vec![0.0; points.len()];
    let mut l = vec![1.0; points.len()];
    let mut mu = vec![0.0; points.len()];
    let mut z = vec![0.0; points.len()];

    for i in 1..n {
        l[i] = 2.0 * (points[i + 1] - points[i - 1]) - h[i - 1] * mu[i - 1];
        mu[i] = h[i] / l[i];
        z[i] = (alpha[i] - h[i - 1] * z[i - 1]) / l[i];
    }

    for i in (0..n).rev() {
        c[i] = z[i] - mu[i] * c[i + 1];
        b[i] = (a[i + 1] - a[i]) / h[i] - h[i] * (c[i + 1] + 2.0 * c[i]) / 3.0;
        d[i] = (c[i + 1] - c[i]) / (3.0 * h[i]);
    }

    let mut coefficients = Vec::new();
    for i in 0..n {
        coefficients.push([a[i], b[i], c[i], d[i]]);
    }

    coefficients
}
fn evaluate_cubic(coefficients: &[f32; 4], t: f32) -> f32 {
    coefficients[0] + coefficients[1] * t + coefficients[2] * t * t + coefficients[3] * t * t * t
}
