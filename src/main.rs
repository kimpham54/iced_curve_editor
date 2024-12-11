use iced::widget::{canvas, container, hover, Canvas, button, horizontal_space};
// use iced::{border, mouse, Color, Length, Point, Rectangle, Renderer, Size};
use iced::{Element, Fill, Theme};

pub fn main() -> iced::Result {
    // Entry point of the application. This initializes and runs the application.
    // `iced::application` creates an application instance, passing the update and view methods.
    iced::application(
        "Curve Editor Tool - Iced",
        ExampleCanvas::update,
        ExampleCanvas::view,
    )
    .theme(|_| Theme::CatppuccinMocha)
    .run() // Start the application.
}

#[derive(Default)]
struct ExampleCanvas {
    drawdot: DotState,
    dots: Vec<Dot>, // Stores all drawn points.
}

impl ExampleCanvas {
    fn update(&mut self, message: Message) {
        match message {
            Message::AddDot(dot) => {
                self.dots.push(dot);
                dbg!(&self.dots);
                self.drawdot.request_redraw();

            }
            Message::Clear => {
                self.drawdot = DotState::default();
                self.dots.clear()
            }
        }
    }

    /// Builds the user interface (UI) for the application.
    fn view(&self) -> Element<Message> {
        container(hover(
            self.drawdot.view(&self.dots).map(Message::AddDot),
            if self.dots.is_empty() {
                container(horizontal_space())
            } else {
                container(
                    button("Clear")
                        .style(button::danger)
                        .on_press(Message::Clear),
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
    AddDot(Dot), // Message to add a new curve.
    Clear,               // Message to clear all points.
}

struct DrawDot<'a> {
    state: &'a DotState,
    dots: &'a [Dot], // List of points to render.
}

#[derive(Default)]
struct DotState {
    cache: canvas::Cache,
}

impl DotState {
    pub fn view<'a>(&'a self, dots: &'a [Dot]) -> Element<'a, Dot> {
        Canvas::new(DrawDot { state: self, dots })
            .width(Fill)
            .height(Fill)
            .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear();
    }
}

impl canvas::Program<Dot> for DrawDot<'_> {
    type State = DotState;

    /// Handles events on the canvas, such as mouse clicks.
    fn update(
        &self,
        _state: &mut Self::State,
        event: iced::widget::canvas::event::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
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
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let content = self.state.cache.draw(renderer, bounds.size(), |frame| {
            // Iterate through all the dots and draw them on the canvas.
            for dot in self.dots {
                // Directly use the x and y fields of the iced::Point.
                // Draw a filled circle at the position of the dot.
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
        });

        vec![content]
    }
}

#[derive(Debug, Clone, Copy)]
struct Dot {
    position: iced::Point,
}
