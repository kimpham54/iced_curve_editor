use iced::widget::{button, canvas, container, horizontal_space, hover, Canvas};
// use iced::{border, Color, Length, Renderer, Size};
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
}

impl ExampleCanvas {
    fn update(&mut self, message: Message) {
        match message {
            Message::AddDot(dot) => {
                self.dots.push(dot);
                self.dotstate.request_redraw();
                // dbg!(&self.dots);
                if self.straight_mode {
                    self.dotstate.request_redraw(); // Ensure lines are regenerated
                }
            }
            Message::Clear => {
                self.dotstate = DotState::default();
                self.dots.clear();
                self.straight_mode = false; // Reset the mode
            }
            Message::Straight => {
                self.straight_mode = !self.straight_mode; // Toggle the mode
                self.dotstate.request_redraw(); // Redraw to show/hide lines
            }
            Message::Curve => {
                // todo: replace placeholder functionality for curve line creation
            }
        }
    }

    /// Builds the user interface (UI) for the application.
    fn view(&self) -> Element<Message> {
        container(hover(
            self.dotstate
                .view(&self.dots, self.straight_mode)
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
                        button("Curve").on_press(Message::Curve)
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
enum Message {
    AddDot(Dot), // Message to add a new point.
    Clear,       // Message to clear all points.
    Straight,    // Toggle straight line connector mode on and off.
    Curve,       // Toggle curve line connector mode between cubic bezier, catmull rom splines, and off.
}

struct DrawDotsAndLines<'a> {
    state: &'a DotState,
    dots: &'a [Dot],
    straight_mode: bool,
}

#[derive(Default)]
struct DotState {
    cache: canvas::Cache,
}

impl DotState {
    pub fn view<'a>(&'a self, dots: &'a [Dot], straight_mode: bool) -> Element<'a, Dot> {
        Canvas::new(DrawDotsAndLines {
            state: self,
            dots,
            straight_mode,
        }) //Pass straight_mode to DrawDotsandLines
        .width(Fill)
        .height(Fill)
        .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear();
    }
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
            // Draw border
            frame.stroke(
                &Path::rectangle(Point::ORIGIN, frame.size()),
                Stroke::default()
                    .with_width(2.0)
                    .with_color(theme.palette().text),
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

            // Draw straight line connectors if "Straight" is active
            if self.straight_mode {
                let mut sorted_dots = self.dots.to_vec();
                sorted_dots.sort_by(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap());

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
        });

        vec![content]
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
