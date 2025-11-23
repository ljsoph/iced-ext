use iced_core::Background;
use iced_core::Border;
use iced_core::Clipboard;
use iced_core::Color;
use iced_core::Element;
use iced_core::Length;
use iced_core::Pixels;
use iced_core::Rectangle;
use iced_core::Shell;
use iced_core::Size;
use iced_core::Theme;
use iced_core::alignment;
use iced_core::layout::Layout;
use iced_core::layout::{self};
use iced_core::mouse;
use iced_core::mouse::Button;
use iced_core::renderer;
use iced_core::text;
use iced_core::widget::Tree;
use iced_core::widget::Widget;
use iced_core::widget::tree;
use iced_core::widget::{self};
use iced_core::{self};

pub struct SquareRadio<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    is_selected: bool,
    on_click: Message,
    size: f32,
    width: Length,
    label: Option<String>,
    spacing: Option<f32>,
    last_status: Option<Status>,
    icon: Icon<Renderer::Font>,
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    text_wrapping: text::Wrapping,
    font: Option<Renderer::Font>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> SquareRadio<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    const DEFAULT_SIZE: f32 = 16.0;
    const DEFAULT_SPACING: f32 = 8.0;

    pub fn new<V, F>(value: V, selection: Option<V>, f: F) -> Self
    where
        F: FnOnce(V) -> Message,
        V: Eq + Copy,
    {
        Self {
            is_selected: Some(value) == selection,
            on_click: f(value),
            size: Self::DEFAULT_SIZE,
            width: Length::Shrink,
            label: None,
            spacing: None,
            last_status: None,
            icon: Icon {
                font: Renderer::ICON_FONT,
                code_point: Renderer::CHECKMARK_ICON,
                size: None,
                line_height: text::LineHeight::default(),
                shaping: text::Shaping::Basic,
            },
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_shaping: text::Shaping::default(),
            text_wrapping: text::Wrapping::default(),
            font: None,
            class: Theme::default(),
        }
    }

    /// Sets the width of the [`SquareRadio`] button.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the text label of the [`SquareRadio`]
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the spacing between the [`SquareRadio`] and text.
    #[must_use]
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = Some(spacing.into().0);
        self
    }

    /// Sets the text size of the [`SquareRadio`] label.
    #[must_use]
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into());
        self
    }

    /// Sets the text [`text::LineHeight`] of the label.
    #[must_use]
    pub fn text_line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.text_line_height = line_height.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for SquareRadio<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<widget::text::State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(widget::text::State::<Renderer::Paragraph>::default())
    }

    fn layout(&mut self, tree: &mut widget::Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        if let Some(label) = &self.label {
            layout::next_to_each_other(
                &limits.width(self.width),
                self.spacing.unwrap_or(Self::DEFAULT_SPACING),
                |_| layout::Node::new([self.size, self.size].into()),
                |limits| {
                    let state = tree.state.downcast_mut::<widget::text::State<Renderer::Paragraph>>();

                    widget::text::layout(
                        state,
                        renderer,
                        limits,
                        label,
                        widget::text::Format {
                            width: self.width,
                            height: Length::Shrink,
                            line_height: self.text_line_height,
                            size: self.text_size,
                            font: self.font,
                            align_x: text::Alignment::Default,
                            align_y: alignment::Vertical::Center,
                            shaping: self.text_shaping,
                            wrapping: self.text_wrapping,
                        },
                    )
                },
            )
        } else {
            layout::Node::new([self.size, self.size].into())
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let box_layout = if self.label.is_none() {
            layout
        } else {
            layout.children().next().unwrap()
        };

        let box_bounds = box_layout.bounds();

        let style = theme.style(
            &self.class,
            self.last_status.unwrap_or(Status::Active {
                is_selected: self.is_selected,
            }),
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: box_bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        let Icon {
            font,
            code_point,
            size,
            line_height,
            shaping,
        } = &self.icon;
        let size = size.unwrap_or(Pixels(box_bounds.height * 0.7));

        if self.is_selected {
            renderer.fill_text(
                text::Text {
                    content: code_point.to_string(),
                    font: *font,
                    size,
                    line_height: *line_height,
                    bounds: box_bounds.size(),
                    align_x: text::Alignment::Center,
                    align_y: alignment::Vertical::Center,
                    shaping: *shaping,
                    wrapping: text::Wrapping::default(),
                },
                box_bounds.center(),
                style.icon_color,
                *viewport,
            );
        }

        if self.label.is_some() {
            let label_layout = layout.child(1);
            let label_bounds = label_layout.bounds();
            let state: &widget::text::State<Renderer::Paragraph> = tree.state.downcast_ref();
            widget::text::draw(
                renderer,
                &renderer::Style::default(),
                label_bounds,
                state.raw(),
                widget::text::Style::default(),
                viewport,
            );
        }
    }

    fn update(
        &mut self,
        _state: &mut widget::Tree,
        event: &iced_core::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if let iced_core::Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) = event
            && cursor.is_over(layout.bounds())
        {
            shell.publish(self.on_click.clone());
            shell.capture_event();
        }
    }

    fn operate(
        &mut self,
        _state: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        if let Some(label) = &self.label {
            operation.text(None, layout.bounds(), label);
        }
    }
}

impl<'a, Message, Theme, Renderer> From<SquareRadio<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(widget: SquareRadio<'a, Message, Theme, Renderer>) -> Self {
        Self::new(widget)
    }
}

/// The icon in a [`SquareRadio`].
#[derive(Debug, Clone, PartialEq)]
pub struct Icon<Font> {
    pub font: Font,
    pub code_point: char,
    pub size: Option<Pixels>,
    pub line_height: text::LineHeight,
    pub shaping: text::Shaping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active { is_selected: bool },
    Hovered { is_selected: bool },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub background: Background,
    pub icon_color: Color,
    pub border: Border,
    pub text_color: Option<Color>,
}

pub trait Catalog {
    type Class<'a>;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`SquareRadio`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`SquareRadio`] button.
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let active = Style {
        background: Color::TRANSPARENT.into(),
        icon_color: palette.primary.strong.color,
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 2.0.into(),
        },
        text_color: None,
    };

    match status {
        Status::Active { .. } => active,
        Status::Hovered { is_selected } => {
            let (background, border) = if is_selected {
                (palette.background.strong, palette.primary.strong.color)
            } else {
                (palette.background.weak, palette.background.strong.color)
            };
            Style {
                icon_color: palette.primary.strong.color,
                background: Background::Color(background.color),
                border: Border {
                    color: border,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..active
            }
        }
    }
}
