//! Progress bars visualize the progression of an extended computer operation, such as a download, file transfer, or installation.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! #
//! use iced::widget::progress_bar;
//!
//! struct State {
//!    progress: f32,
//! }
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     progress_bar(0.0..=100.0, state.progress).into()
//! }
//! ```
use core::f32;
use std::ops::RangeInclusive;

use iced_core::Background;
use iced_core::Color;
use iced_core::Element;
use iced_core::Layout;
use iced_core::Length;
use iced_core::Padding;
use iced_core::Pixels;
use iced_core::Point;
use iced_core::Rectangle;
use iced_core::Size;
use iced_core::Text;
use iced_core::Theme;
use iced_core::Widget;
use iced_core::alignment;
use iced_core::border::Border;
use iced_core::border::{self};
use iced_core::layout;
use iced_core::mouse;
use iced_core::renderer;
use iced_core::text;
use iced_core::widget::Tree;
use iced_core::{self};

/// A bar that displays progress.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::progress_bar;
///
/// struct State {
///    progress: f32,
/// }
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     progress_bar(0.0..=100.0, state.progress).into()
/// }
/// ```
pub struct ProgressBar<'a, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    range: RangeInclusive<f32>,
    value: f32,
    length: Length,
    girth: Length,
    is_vertical: bool,
    show_percentage: bool,
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    padding: Padding,
    alignment: alignment::Horizontal,
    font: Option<Renderer::Font>,
    class: Theme::Class<'a>,
}

impl<'a, Theme, Renderer> ProgressBar<'a, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// The default girth of a [`ProgressBar`].
    pub const DEFAULT_GIRTH: f32 = 30.0;

    /// Creates a new [`ProgressBar`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`ProgressBar`]
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        ProgressBar {
            value: value.clamp(*range.start(), *range.end()),
            range,
            length: Length::Fill,
            girth: Length::from(Self::DEFAULT_GIRTH),
            is_vertical: false,
            show_percentage: true,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            padding: Padding::ZERO,
            alignment: alignment::Horizontal::Left,
            font: None,
            class: Theme::default(),
        }
    }

    /// Sets the width of the [`ProgressBar`].
    pub fn length(mut self, length: impl Into<Length>) -> Self {
        self.length = length.into();
        self
    }

    /// Sets the height of the [`ProgressBar`].
    pub fn girth(mut self, girth: impl Into<Length>) -> Self {
        self.girth = girth.into();
        self
    }

    /// Turns the [`ProgressBar`] into a vertical [`ProgressBar`].
    ///
    /// By default, a [`ProgressBar`] is horizontal.
    pub fn vertical(mut self) -> Self {
        self.is_vertical = true;
        self
    }

    /// Sets the style of the [`ProgressBar`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`ProgressBar`].
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the text size of the [`ProgressBar`] current value.
    #[must_use]
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into());
        self
    }

    /// Sets the text [`text::LineHeight`] of the current value.
    #[must_use]
    pub fn text_line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the [`Padding`] of the current value.
    #[must_use]
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the horizontal alignment of the value within the [`ProgressBar`].
    ///
    /// By default, the value is  [`Horizontal::Left`].
    #[must_use]
    pub fn alignment(mut self, alignment: alignment::Horizontal) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets the [`Padding`] of the current value.
    #[must_use]
    pub fn percentage(mut self, show_percentage: bool) -> Self {
        self.show_percentage = show_percentage;
        self
    }

    /// Show the current percentage on the [`ProgressBar`].
    ///
    /// By default, the percentage is shown.
    #[must_use]
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    fn width(&self) -> Length {
        if self.is_vertical { self.girth } else { self.length }
    }

    fn height(&self) -> Length {
        if self.is_vertical { self.length } else { self.girth }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for ProgressBar<'_, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, self.width(), self.height())
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let (range_start, range_end) = self.range.clone().into_inner();
        let length = if self.is_vertical { bounds.height } else { bounds.width };
        let active_progress_length = if range_start >= range_end {
            0.0
        } else {
            length * (self.value - range_start) / (range_end - range_start)
        };

        let style = theme.style(&self.class);

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle { ..bounds },
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        if active_progress_length > 0.0 {
            let bounds = if self.is_vertical {
                Rectangle {
                    y: bounds.y + bounds.height - active_progress_length,
                    height: active_progress_length,
                    ..bounds
                }
            } else {
                Rectangle {
                    width: active_progress_length,
                    ..bounds
                }
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        color: Color::TRANSPARENT,
                        ..style.border
                    },
                    ..renderer::Quad::default()
                },
                style.bar,
            );
        }

        if self.show_percentage {
            let (x, align_x) = match self.alignment {
                alignment::Horizontal::Left => (bounds.x + self.padding.left, text::Alignment::Left),
                alignment::Horizontal::Center => (bounds.x + (bounds.width / 2.0), text::Alignment::Center),
                alignment::Horizontal::Right => (bounds.x + bounds.width - self.padding.right, text::Alignment::Right),
            };
            renderer.fill_text(
                Text {
                    content: format!("{}%", self.value),
                    bounds: Size::new(f32::INFINITY, bounds.height),
                    size: self.text_size.unwrap_or_else(|| renderer.default_size()),
                    line_height: self.text_line_height,
                    font: self.font.unwrap_or_else(|| renderer.default_font()),
                    align_x,
                    align_y: alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::default(),
                },
                Point::new(x, bounds.center_y()),
                theme.style(&self.class).color,
                *viewport,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<ProgressBar<'a, Theme, Renderer>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + text::Renderer,
{
    fn from(progress_bar: ProgressBar<'a, Theme, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(progress_bar)
    }
}

/// The appearance of a progress bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the progress bar.
    pub background: Background,
    /// The [`Background`] of the bar of the progress bar.
    pub bar: Background,
    /// The [`Border`] of the progress bar.
    pub border: Border,
    /// The [`Color`] of the progress bar percentage.
    pub color: Color,
}

/// The theme catalog of a [`ProgressBar`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`ProgressBar`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The primary style of a [`ProgressBar`].
pub fn primary(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(
        palette.background.strong.color,
        palette.primary.base.color,
        palette.background.strongest.text,
    )
}

/// The secondary style of a [`ProgressBar`].
pub fn secondary(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(
        palette.background.strong.color,
        palette.secondary.base.color,
        palette.background.weak.text,
    )
}

/// The success style of a [`ProgressBar`].
pub fn success(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(
        palette.background.strong.color,
        palette.success.base.color,
        palette.background.weak.text,
    )
}

/// The warning style of a [`ProgressBar`].
pub fn warning(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(
        palette.background.strong.color,
        palette.warning.base.color,
        palette.background.weak.text,
    )
}

/// The danger style of a [`ProgressBar`].
pub fn danger(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(
        palette.background.strong.color,
        palette.danger.base.color,
        palette.background.weak.text,
    )
}

fn styled(background: impl Into<Background>, bar: impl Into<Background>, color: Color) -> Style {
    Style {
        background: background.into(),
        bar: bar.into(),
        border: border::rounded(2),
        color,
    }
}
