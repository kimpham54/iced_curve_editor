// IMPLEMENTS ALGORITHM MANUALLY
use iced::widget::{button, canvas, container, horizontal_space, hover, Canvas};
use iced::widget::canvas::{Path, Stroke};
use iced::{mouse, Element, Fill, Point, Rectangle, Theme};

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
}

#[derive(Debug, Clone, Copy)]
enum CurveAlgorithm {
    CatmullRom,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    AddDot(Dot), // Message to add a new point.
    Clear,       // Message to clear all points.
    Straight,    // Toggle straight line connector mode on and off.
    Curve, // Toggle curve line connector mode between catmull rom splines and off, can add more in future if needed
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
                    Some(CurveAlgorithm::CatmullRom) => None, // 2nd press
                };
                self.dotstate.request_redraw();
            }
        }
    }

    /// Builds the user interface (UI) for the application.
    fn view(&self) -> Element<Message> {
        container(hover(
            self.dotstate
                .view(&self.dots, self.straight_mode, self.curve_mode)
                .map(Message::AddDot),
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
                        if self.dots.len() >= 2 { // Only enable curve button if there are >2 points
                            button(match self.curve_mode {
                                None => "Curve: Off",
                                Some(CurveAlgorithm::CatmullRom) => "Curve: Catmull-Rom",
                            })
                            .on_press(Message::Curve) // Button is active
                        } else {
                            button("Curve: Disabled") // Button is disabled (no `on_press`)
                        }
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
    ) -> Element<'a, Dot> {
        Canvas::new(DrawDotsAndLines {
            state: self,
            dots,
            straight_mode,
            curve_mode,
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
        match event {
            iced::widget::canvas::event::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            )) => {
                let dot = Dot {
                    position: cursor_position,
                };
                (iced::widget::canvas::event::Status::Captured, Some(dot))
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
            }}

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



 // Draw Catmull-Rom or Bézier curves
 if let Some(curve_mode) = self.curve_mode {
    let mut path = Path::new(|builder| {
        let n_points = 200; // Number of interpolated points

        for i in 0..n_points {
            let t = i as f32 / (n_points - 1) as f32;
            let segment_index = ((sorted_dots.len() - 1) as f32 * t).floor() as usize;

            // Clamp control points for safe interpolation
            let p0 = if segment_index == 0 {
                &sorted_dots[segment_index]
            } else {
                &sorted_dots[segment_index - 1]
            };
            let p1 = &sorted_dots[segment_index];
            let p2 = &sorted_dots[std::cmp::min(segment_index + 1, sorted_dots.len() - 1)];
            let p3 = &sorted_dots[std::cmp::min(segment_index + 2, sorted_dots.len() - 1)];

            let local_t = (t * (sorted_dots.len() - 1) as f32) % 1.0;

            let (x, y) = match curve_mode {
                CurveAlgorithm::CatmullRom => {
                    let x = catmull_rom_centripetal(local_t, p0.position.x, p1.position.x, p2.position.x, p3.position.x, 0.5);
                    let y = catmull_rom_centripetal(local_t, p0.position.y, p1.position.y, p2.position.y, p3.position.y, 0.5);
                    (x, y)
                }
                // CurveAlgorithm::Bezier => {
                //     let x = bezier(local_t, p0.position.x, p1.position.x, p2.position.x, p3.position.x);
                //     let y = bezier(local_t, p0.position.y, p1.position.y, p2.position.y, p3.position.y);
                //     (x, y)
                // }
            };

            if i == 0 {
                builder.move_to(Point { x, y });
            } else {
                builder.line_to(Point { x, y });
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
}
        );

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

fn catmull_rom_centripetal(
    t: f32, p0: f32, p1: f32, p2: f32, p3: f32, alpha: f32) -> f32 {
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