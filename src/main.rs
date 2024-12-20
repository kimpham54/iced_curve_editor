use iced::widget::{button, canvas, container, horizontal_space, hover, Canvas};
// use iced::{border, Color, Length, Renderer, Size};
use iced::widget::canvas::{Path, Stroke};
use iced::{mouse, Element, Fill, Point, Rectangle, Theme};
use uniform_cubic_splines::{basis::Bezier, basis::CatmullRom, basis::Linear, spline, spline_inverse};

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
    Linear,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    AddDot(Dot), // Message to add a new point.
    Clear,       // Message to clear all points.
    Straight,    // Toggle straight line connector mode on and off.
    Curve, // Toggle curve line connector mode between linear, catmull rom splines, and off.
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
                // Cycle through curve modes: Catmull-Rom -> Cubic Bézier -> Off
                self.curve_mode = match self.curve_mode {
                    // None => Some(CurveAlgorithm::CatmullRom), // 1st press
                    None => Some(CurveAlgorithm::Linear), // 1st press
                    Some(CurveAlgorithm::Linear) => Some(CurveAlgorithm::CatmullRom), // 2nd press
                    Some(CurveAlgorithm::CatmullRom) => None, // 3rd press
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
                        if self.dots.len() >= 4 {
                            button(match self.curve_mode {
                                None => "Curve: Off",
                                Some(CurveAlgorithm::CatmullRom) => "Curve: Catmull-Rom",
                                Some(CurveAlgorithm::Linear) => "Curve: Linear",
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
                let dot_test1 = Dot {
                    position: Point { x: 0.0, y: 0.0 },
                };
                let dot_test2 = Dot {
                    position: Point {
                        x: bounds_width,
                        y: 0.0,
                    },
                };
                sorted_dots.push(first_control_dot);
                sorted_dots.push(last_control_dot);
                // sorted_dots.push(dot_test1);
                // sorted_dots.push(dot_test2);
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

            // Draw curve line connector
            if let Some(curve_mode) = self.curve_mode {
                let x_knots: Vec<f32> = sorted_dots.iter().map(|dot| dot.position.x).collect();
                let y_knots: Vec<f32> = sorted_dots.iter().map(|dot| dot.position.y).collect();
                let knot_spacing: Vec<f32> = (0..x_knots.len())
                    .map(|i| i as f32 / (x_knots.len() - 1) as f32)
                    .collect();

                // Build the curve path
                let mut path = Path::new(|builder| {
                    let n_points = 500; // Number of points to interpolate
                    for i in 0..=n_points {
                        let t = i as f32 / n_points as f32; // t ranges from 0.0 to 1.0

                        // Map t to the spline parameter using spline_inverse
                        if let (Some(vx), Some(vy)) = (
                            spline_inverse::<CatmullRom, _>(t, &knot_spacing, None, None),
                            spline_inverse::<CatmullRom, _>(t, &knot_spacing, None, None),
                        ) {
                            
                            // Compute the x and y positions using the spline
                            let (x, y) = match curve_mode {
                                CurveAlgorithm::CatmullRom => {
                                    let x = spline::<CatmullRom, _, _>(vx, &x_knots);
                                    let y = spline::<CatmullRom, _, _>(vy, &y_knots);
                                    (x, y)
                                } CurveAlgorithm::Linear => {
                                  let x = spline::<Linear, _, _>(vx, &x_knots);
                                  let y = spline::<Linear, _, _>(vy, &y_knots);
                                  (x, y)
                                  }
                            };

                            if i == 0 {
                                builder.move_to(Point { x, y });
                            } else {
                                builder.line_to(Point { x, y });
                            }
                        }
                    }
                });

                // Draw the curve
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
