use iced_core::alignment;
use iced_core::keyboard;
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::text::paragraph;
use iced_core::text::{self, Text};
use iced_core::touch;
use iced_core::widget::tree::{self, Tree};
use iced_core::window;
use iced_core::{
    Background, Border, Clipboard, Color, Element, Event, Layout, Length, Padding, Pixels, Point, Rectangle, Shell,
    Size, Theme, Vector, Widget,
};

use std::borrow::Borrow;
use std::f32;

pub struct MultiPickList<'a, T, L, V, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone,
    L: Borrow<[T]> + 'a,
    V: Borrow<[T]> + 'a,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    options: L,
    label: Option<String>,
    selected: V,
    width: Length,
    padding: Padding,
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    handle: Handle<Renderer::Font>,
    class: <Theme as Catalog>::Class<'a>,
    menu_class: <Theme as menu::Catalog>::Class<'a>,
    last_status: Option<Status>,
    menu_height: Length,
}

impl<'a, T, L, V, Message, Theme, Renderer> MultiPickList<'a, T, L, V, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone,
    L: Borrow<[T]> + 'a,
    V: Borrow<[T]> + 'a,
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    pub fn new(options: L, selected: V, on_select: impl Fn(T) -> Message + 'a) -> Self {
        Self {
            on_select: Box::new(on_select),
            on_open: None,
            on_close: None,
            options,
            label: None,
            selected,
            width: Length::Shrink,
            padding: iced_widget::button::DEFAULT_PADDING,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_shaping: text::Shaping::default(),
            font: None,
            handle: Handle::default(),
            class: <Theme as Catalog>::default(),
            menu_class: <Theme as Catalog>::default_menu(),
            last_status: None,
            menu_height: Length::Shrink,
        }
    }

    /// Sets the placeholder of the [`MultiPickList`].
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the width of the [`MultiPickList`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Menu`].
    pub fn menu_height(mut self, menu_height: impl Into<Length>) -> Self {
        self.menu_height = menu_height.into();
        self
    }

    /// Sets the [`Padding`] of the [`MultiPickList`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`MultiPickList`].
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = Some(size.into());
        self
    }

    /// Sets the text [`text::LineHeight`] of the [`MultiPickList`].
    pub fn text_line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`MultiPickList`].
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    /// Sets the font of the [`MultiPickList`].
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the [`Handle`] of the [`MultiPickList`].
    pub fn handle(mut self, handle: Handle<Renderer::Font>) -> Self {
        self.handle = handle;
        self
    }

    /// Sets the message that will be produced when the [`MultiPickList`] is opened.
    pub fn on_open(mut self, on_open: Message) -> Self {
        self.on_open = Some(on_open);
        self
    }

    /// Sets the message that will be produced when the [`MultiPickList`] is closed.
    pub fn on_close(mut self, on_close: Message) -> Self {
        self.on_close = Some(on_close);
        self
    }

    /// Sets the style of the [`MultiPickList`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style of the [`Menu`].
    #[must_use]
    pub fn menu_style(mut self, style: impl Fn(&Theme) -> menu::Style + 'a) -> Self
    where
        <Theme as menu::Catalog>::Class<'a>: From<menu::StyleFn<'a, Theme>>,
    {
        self.menu_class = (Box::new(style) as menu::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`MultiPickList`].
    #[must_use]
    pub fn class(mut self, class: impl Into<<Theme as Catalog>::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the style class of the [`Menu`].
    #[must_use]
    pub fn menu_class(mut self, class: impl Into<<Theme as menu::Catalog>::Class<'a>>) -> Self {
        self.menu_class = class.into();
        self
    }
}

impl<'a, T, L, V, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for MultiPickList<'a, T, L, V, Message, Theme, Renderer>
where
    T: Clone + ToString + PartialEq + 'a,
    L: Borrow<[T]>,
    V: Borrow<[T]>,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
        let options = self.options.borrow();

        state.options.resize_with(options.len(), Default::default);

        let option_text = Text {
            content: "",
            bounds: Size::new(f32::INFINITY, self.text_line_height.to_absolute(text_size).into()),
            size: text_size,
            line_height: self.text_line_height,
            font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Center,
            shaping: self.text_shaping,
            wrapping: text::Wrapping::default(),
        };

        for (option, paragraph) in options.iter().zip(state.options.iter_mut()) {
            let label = option.to_string();

            let _ = paragraph.update(Text {
                content: &label,
                ..option_text
            });
        }

        if let Some(label) = &self.label {
            let _ = state.label.update(Text {
                content: label,
                ..option_text
            });
        }

        let max_width = match self.width {
            Length::Shrink => {
                let labels_width = state
                    .options
                    .iter()
                    .fold(0.0, |width, paragraph| f32::max(width, paragraph.min_width()));

                labels_width.max(self.label.as_ref().map(|_| state.label.min_width()).unwrap_or(0.0))
            }
            _ => 0.0,
        };

        let size = {
            let intrinsic = Size::new(
                max_width + text_size.0 + self.padding.left,
                f32::from(self.text_line_height.to_absolute(text_size)),
            );

            limits
                .width(self.width)
                .shrink(self.padding)
                .resolve(self.width, Length::Shrink, intrinsic)
                .expand(self.padding)
        };

        layout::Node::new(size)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if state.is_open {
                    // Event wasn't processed by overlay, so cursor was clicked either outside its
                    // bounds or on the drop-down, either way we close the overlay.
                    state.is_open = false;
                    state.hovered_option = None;

                    if let Some(on_close) = &self.on_close {
                        shell.publish(on_close.clone());
                    }

                    shell.capture_event();
                } else if cursor.is_over(layout.bounds()) {
                    state.is_open = true;

                    if let Some(on_open) = &self.on_open {
                        shell.publish(on_open.clone());
                    }

                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.keyboard_modifiers = *modifiers;
            }
            _ => {}
        };

        let status = {
            let is_hovered = cursor.is_over(layout.bounds());

            if state.is_open {
                Status::Opened { is_hovered }
            } else if is_hovered {
                Status::Hovered
            } else {
                Status::Active
            }
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.last_status = Some(status);
        } else if self.last_status.is_some_and(|last_status| last_status != status) {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let is_mouse_over = cursor.is_over(bounds);

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let bounds = layout.bounds();
        let style = Catalog::style(theme, &self.class, self.last_status.unwrap_or(Status::Active));

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        let handle = match &self.handle {
            Handle::Arrow { size } => Some((
                Renderer::ICON_FONT,
                Renderer::ARROW_DOWN_ICON,
                *size,
                text::LineHeight::default(),
                text::Shaping::Basic,
            )),
            Handle::Static(Icon {
                font,
                code_point,
                size,
                line_height,
                shaping,
            }) => Some((*font, *code_point, *size, *line_height, *shaping)),
            Handle::Dynamic { open, closed } => {
                if state.is_open {
                    Some((open.font, open.code_point, open.size, open.line_height, open.shaping))
                } else {
                    Some((
                        closed.font,
                        closed.code_point,
                        closed.size,
                        closed.line_height,
                        closed.shaping,
                    ))
                }
            }
            Handle::None => None,
        };

        if let Some((font, code_point, size, line_height, shaping)) = handle {
            let size = size.unwrap_or_else(|| renderer.default_size());

            renderer.fill_text(
                Text {
                    content: code_point.to_string(),
                    size,
                    line_height,
                    font,
                    bounds: Size::new(bounds.width, f32::from(line_height.to_absolute(size))),
                    align_x: text::Alignment::Right,
                    align_y: alignment::Vertical::Center,
                    shaping,
                    wrapping: text::Wrapping::default(),
                },
                Point::new(bounds.x + bounds.width - self.padding.right, bounds.center_y()),
                style.handle_color,
                *viewport,
            );
        }

        if let Some(label) = &self.label {
            renderer.fill_text(
                Text {
                    content: label.clone(),
                    bounds: Size::new(f32::INFINITY, bounds.height),
                    size: self.text_size.unwrap_or_else(|| renderer.default_size()),
                    line_height: self.text_line_height,
                    font: self.font.unwrap_or_else(|| renderer.default_font()),
                    align_x: text::Alignment::Left,
                    align_y: alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::default(),
                },
                Point::new(bounds.x + self.padding.left, bounds.center_y()),
                style.text_color,
                *viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        if state.is_open {
            let bounds = layout.bounds();

            let on_select = &self.on_select;

            let mut menu = menu::Menu::new(
                &mut state.menu,
                self.options.borrow(),
                self.selected.borrow(),
                &mut state.hovered_option,
                |option| {
                    // We don't want to close the window if we are selecting multiple
                    // things at once
                    // state.is_open = false;

                    (on_select)(option)
                },
                None,
                &self.menu_class,
            )
            .width(bounds.width)
            .padding(self.padding)
            .font(font)
            .text_shaping(self.text_shaping);

            if let Some(text_size) = self.text_size {
                menu = menu.text_size(text_size);
            }

            Some(menu.overlay(
                layout.position() + translation,
                *viewport,
                bounds.height,
                self.menu_height,
            ))
        } else {
            None
        }
    }
}

impl<'a, T, L, V, Message, Theme, Renderer> From<MultiPickList<'a, T, L, V, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: Clone + ToString + PartialEq + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<[T]> + 'a,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(pick_list: MultiPickList<'a, T, L, V, Message, Theme, Renderer>) -> Self {
        Self::new(pick_list)
    }
}

#[derive(Debug)]
struct State<P: text::Paragraph> {
    menu: menu::State,
    keyboard_modifiers: keyboard::Modifiers,
    is_open: bool,
    hovered_option: Option<usize>,
    options: Vec<paragraph::Plain<P>>,
    label: paragraph::Plain<P>,
}

impl<P: text::Paragraph> State<P> {
    /// Creates a new [`State`] for a [`MultiPickList`].
    fn new() -> Self {
        Self {
            menu: menu::State::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_open: bool::default(),
            hovered_option: Option::default(),
            options: Vec::new(),
            label: paragraph::Plain::default(),
        }
    }
}

impl<P: text::Paragraph> Default for State<P> {
    fn default() -> Self {
        Self::new()
    }
}

/// The handle to the right side of the [`MultiPickList`].
#[derive(Debug, Clone, PartialEq)]
pub enum Handle<Font> {
    /// Displays an arrow icon (▼).
    ///
    /// This is the default.
    Arrow {
        /// Font size of the content.
        size: Option<Pixels>,
    },
    /// A custom static handle.
    Static(Icon<Font>),
    /// A custom dynamic handle.
    Dynamic {
        /// The [`Icon`] used when [`MultiPickList`] is closed.
        closed: Icon<Font>,
        /// The [`Icon`] used when [`MultiPickList`] is open.
        open: Icon<Font>,
    },
    /// No handle will be shown.
    None,
}

impl<Font> Default for Handle<Font> {
    fn default() -> Self {
        Self::Arrow { size: None }
    }
}

/// The icon of a [`Handle`].
#[derive(Debug, Clone, PartialEq)]
pub struct Icon<Font> {
    /// Font that will be used to display the `code_point`,
    pub font: Font,
    /// The unicode code point that will be used as the icon.
    pub code_point: char,
    /// Font size of the content.
    pub size: Option<Pixels>,
    /// Line height of the content.
    pub line_height: text::LineHeight,
    /// The shaping strategy of the icon.
    pub shaping: text::Shaping,
}

/// The possible status of a [`MultiPickList`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`MultiPickList`] can be interacted with.
    Active,
    /// The [`MultiPickList`] is being hovered.
    Hovered,
    /// The [`MultiPickList`] is open.
    Opened {
        /// Whether the [`MultiPickList`] is hovered, while open.
        is_hovered: bool,
    },
}

/// The appearance of a pick list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The text [`Color`] of the pick list.
    pub text_color: Color,
    /// The placeholder [`Color`] of the pick list.
    pub placeholder_color: Color,
    /// The handle [`Color`] of the pick list.
    pub handle_color: Color,
    /// The [`Background`] of the pick list.
    pub background: Background,
    /// The [`Border`] of the pick list.
    pub border: Border,
}

/// The theme catalog of a [`MultiPickList`].
pub trait Catalog: menu::Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> <Self as Catalog>::Class<'a>;

    /// The default class for the menu of the [`MultiPickList`].
    fn default_menu<'a>() -> <Self as menu::Catalog>::Class<'a> {
        <Self as menu::Catalog>::default()
    }

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &<Self as Catalog>::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`MultiPickList`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(default)
    }

    fn style(&self, class: &StyleFn<'_, Self>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of the field of a [`MultiPickList`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let active = Style {
        text_color: palette.background.weak.text,
        background: palette.background.weak.color.into(),
        placeholder_color: palette.secondary.base.color,
        handle_color: palette.background.weak.text,
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
    };

    match status {
        Status::Active => active,
        Status::Hovered | Status::Opened { .. } => Style {
            border: Border {
                color: palette.primary.strong.color,
                ..active.border
            },
            ..active
        },
    }
}

pub mod menu {
    //! Build and show dropdown menus.
    use iced_core::border::Border;
    use iced_core::layout::{self, Layout};
    use iced_core::mouse;
    use iced_core::overlay;
    use iced_core::renderer;
    use iced_core::text::{self, Text};
    use iced_core::touch;
    use iced_core::widget::tree::{self, Tree};
    use iced_core::window;
    use iced_core::{
        Background, Clipboard, Color, Event, Length, Padding, Pixels, Point, Rectangle, Shadow, Size, Theme, Vector,
    };
    use iced_core::{Element, Shell, Widget};
    use iced_core::{alignment, border};
    use iced_widget::scrollable::{self, Scrollable};

    /// A list of selectable options.
    pub struct Menu<'a, 'b, T, Message, Theme, Renderer>
    where
        Theme: Catalog,
        Renderer: text::Renderer,
        'b: 'a,
    {
        state: &'a mut State,
        options: &'a [T],
        selected: &'a [T],
        hovered_option: &'a mut Option<usize>,
        on_selected: Box<dyn FnMut(T) -> Message + 'a>,
        on_option_hovered: Option<&'a dyn Fn(T) -> Message>,
        width: f32,
        padding: Padding,
        text_size: Option<Pixels>,
        text_line_height: text::LineHeight,
        text_shaping: text::Shaping,
        font: Option<Renderer::Font>,
        class: &'a <Theme as Catalog>::Class<'b>,
    }

    impl<'a, 'b, T, Message, Theme, Renderer> Menu<'a, 'b, T, Message, Theme, Renderer>
    where
        T: ToString + Clone + PartialEq,
        Message: 'a,
        Theme: Catalog + 'a,
        Renderer: text::Renderer + 'a,
        'b: 'a,
    {
        /// Creates a new [`Menu`] with the given [`State`], a list of options,
        /// the message to produced when an option is selected, and its [`Style`].
        pub fn new(
            state: &'a mut State,
            options: &'a [T],
            selected: &'a [T],
            hovered_option: &'a mut Option<usize>,
            on_selected: impl FnMut(T) -> Message + 'a,
            on_option_hovered: Option<&'a dyn Fn(T) -> Message>,
            class: &'a <Theme as Catalog>::Class<'b>,
        ) -> Self {
            Menu {
                state,
                options,
                selected,
                hovered_option,
                on_selected: Box::new(on_selected),
                on_option_hovered,
                width: 0.0,
                padding: Padding::ZERO,
                text_size: None,
                text_line_height: text::LineHeight::default(),
                text_shaping: text::Shaping::default(),
                font: None,
                class,
            }
        }

        /// Sets the width of the [`Menu`].
        pub fn width(mut self, width: f32) -> Self {
            self.width = width;
            self
        }

        /// Sets the [`Padding`] of the [`Menu`].
        pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
            self.padding = padding.into();
            self
        }

        /// Sets the text size of the [`Menu`].
        pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
            self.text_size = Some(text_size.into());
            self
        }

        /// Sets the text [`text::LineHeight`] of the [`Menu`].
        #[allow(unused)]
        pub fn text_line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
            self.text_line_height = line_height.into();
            self
        }

        /// Sets the [`text::Shaping`] strategy of the [`Menu`].
        pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
            self.text_shaping = shaping;
            self
        }

        /// Sets the font of the [`Menu`].
        pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
            self.font = Some(font.into());
            self
        }

        /// Turns the [`Menu`] into an overlay [`Element`] at the given target
        /// position.
        ///
        /// The `target_height` will be used to display the menu either on top
        /// of the target or under it, depending on the screen position and the
        /// dimensions of the [`Menu`].
        pub fn overlay(
            self,
            position: Point,
            viewport: Rectangle,
            target_height: f32,
            menu_height: Length,
        ) -> overlay::Element<'a, Message, Theme, Renderer> {
            overlay::Element::new(Box::new(Overlay::new(
                position,
                viewport,
                self,
                target_height,
                menu_height,
            )))
        }
    }

    /// The local state of a [`Menu`].
    #[derive(Debug)]
    pub struct State {
        tree: Tree,
    }

    impl State {
        /// Creates a new [`State`] for a [`Menu`].
        pub fn new() -> Self {
            Self { tree: Tree::empty() }
        }
    }

    impl Default for State {
        fn default() -> Self {
            Self::new()
        }
    }

    struct Overlay<'a, 'b, Message, Theme, Renderer>
    where
        Theme: Catalog,
        Renderer: text::Renderer,
    {
        position: Point,
        viewport: Rectangle,
        tree: &'a mut Tree,
        list: Scrollable<'a, Message, Theme, Renderer>,
        width: f32,
        target_height: f32,
        class: &'a <Theme as Catalog>::Class<'b>,
    }

    impl<'a, 'b, Message, Theme, Renderer> Overlay<'a, 'b, Message, Theme, Renderer>
    where
        Message: 'a,
        Theme: Catalog + scrollable::Catalog + 'a,
        Renderer: text::Renderer + 'a,
        'b: 'a,
    {
        pub fn new<T>(
            position: Point,
            viewport: Rectangle,
            menu: Menu<'a, 'b, T, Message, Theme, Renderer>,
            target_height: f32,
            menu_height: Length,
        ) -> Self
        where
            T: Clone + ToString + PartialEq,
        {
            let Menu {
                state,
                options,
                selected,
                hovered_option,
                on_selected,
                on_option_hovered,
                width,
                padding,
                font,
                text_size,
                text_line_height,
                text_shaping,
                class,
            } = menu;

            let list = Scrollable::new(List {
                options,
                selected,
                hovered_option,
                on_selected,
                on_option_hovered,
                font,
                text_size,
                text_line_height,
                text_shaping,
                padding,
                class,
                icon: Icon {
                    font: Renderer::ICON_FONT,
                    code_point: Renderer::CHECKMARK_ICON,
                    size: None,
                    line_height: text::LineHeight::default(),
                    shaping: text::Shaping::Basic,
                },
            })
            .height(menu_height);

            state.tree.diff(&list as &dyn Widget<_, _, _>);

            Self {
                position,
                viewport,
                tree: &mut state.tree,
                list,
                width,
                target_height,
                class,
            }
        }
    }

    impl<Message, Theme, Renderer> iced_core::Overlay<Message, Theme, Renderer>
        for Overlay<'_, '_, Message, Theme, Renderer>
    where
        Theme: Catalog,
        Renderer: text::Renderer,
    {
        fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
            let space_below = bounds.height - (self.position.y + self.target_height);
            let space_above = self.position.y;

            let limits = layout::Limits::new(
                Size::ZERO,
                Size::new(
                    bounds.width - self.position.x,
                    if space_below > space_above {
                        space_below
                    } else {
                        space_above
                    },
                ),
            )
            .width(self.width);

            let node = self.list.layout(self.tree, renderer, &limits);
            let size = node.size();

            node.move_to(if space_below > space_above {
                self.position + Vector::new(0.0, self.target_height)
            } else {
                self.position - Vector::new(0.0, size.height)
            })
        }

        fn update(
            &mut self,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) {
            let bounds = layout.bounds();

            self.list
                .update(self.tree, event, layout, cursor, renderer, clipboard, shell, &bounds);
        }

        fn mouse_interaction(
            &self,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.list
                .mouse_interaction(self.tree, layout, cursor, &self.viewport, renderer)
        }

        fn draw(
            &self,
            renderer: &mut Renderer,
            theme: &Theme,
            defaults: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
        ) {
            let bounds = layout.bounds();

            let style = Catalog::style(theme, self.class);

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    ..renderer::Quad::default()
                },
                style.background,
            );

            self.list
                .draw(self.tree, renderer, theme, defaults, layout, cursor, &bounds);
        }
    }

    struct List<'a, 'b, T, Message, Theme, Renderer>
    where
        Theme: Catalog,
        Renderer: text::Renderer,
    {
        options: &'a [T],
        selected: &'a [T],
        hovered_option: &'a mut Option<usize>,
        on_selected: Box<dyn FnMut(T) -> Message + 'a>,
        on_option_hovered: Option<&'a dyn Fn(T) -> Message>,
        padding: Padding,
        text_size: Option<Pixels>,
        text_line_height: text::LineHeight,
        text_shaping: text::Shaping,
        font: Option<Renderer::Font>,
        class: &'a <Theme as Catalog>::Class<'b>,
        icon: Icon<Renderer::Font>,
    }

    struct ListState {
        is_hovered: Option<bool>,
    }

    impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for List<'_, '_, T, Message, Theme, Renderer>
    where
        T: Clone + ToString + PartialEq,
        Theme: Catalog,
        Renderer: text::Renderer,
    {
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<Option<bool>>()
        }

        fn state(&self) -> tree::State {
            tree::State::new(ListState { is_hovered: None })
        }

        fn size(&self) -> Size<Length> {
            Size {
                width: Length::Fill,
                height: Length::Shrink,
            }
        }

        fn layout(&mut self, _tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
            use std::f32;

            let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
            let text_line_height = self.text_line_height.to_absolute(text_size);
            let size = {
                let intrinsic = Size::new(
                    0.0,
                    (f32::from(text_line_height) + self.padding.y()) * self.options.len() as f32,
                );

                limits.resolve(Length::Fill, Length::Shrink, intrinsic)
            };
            layout::Node::new(size)
        }

        fn update(
            &mut self,
            tree: &mut Tree,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            _clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            _viewport: &Rectangle,
        ) {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if cursor.is_over(layout.bounds())
                        && let Some(index) = *self.hovered_option
                        && let Some(option) = self.options.get(index)
                    {
                        shell.publish((self.on_selected)(option.clone()));
                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());

                        let option_height = f32::from(self.text_line_height.to_absolute(text_size)) + self.padding.y();

                        let new_hovered_option = (cursor_position.y / option_height) as usize;

                        if *self.hovered_option != Some(new_hovered_option)
                            && let Some(option) = self.options.get(new_hovered_option)
                        {
                            if let Some(on_option_hovered) = self.on_option_hovered {
                                shell.publish(on_option_hovered(option.clone()));
                            }

                            shell.request_redraw();
                        }

                        *self.hovered_option = Some(new_hovered_option);
                    }
                }
                Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());

                        let option_height = f32::from(self.text_line_height.to_absolute(text_size)) + self.padding.y();

                        *self.hovered_option = Some((cursor_position.y / option_height) as usize);

                        if let Some(index) = *self.hovered_option
                            && let Some(option) = self.options.get(index)
                        {
                            shell.publish((self.on_selected)(option.clone()));
                            shell.capture_event();
                        }
                    }
                }
                _ => {}
            }

            let state = tree.state.downcast_mut::<ListState>();

            if let Event::Window(window::Event::RedrawRequested(_now)) = event {
                state.is_hovered = Some(cursor.is_over(layout.bounds()));
            } else if state
                .is_hovered
                .is_some_and(|is_hovered| is_hovered != cursor.is_over(layout.bounds()))
            {
                shell.request_redraw();
            }
        }

        fn mouse_interaction(
            &self,
            _tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> mouse::Interaction {
            let is_mouse_over = cursor.is_over(layout.bounds());

            if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            }
        }

        fn draw(
            &self,
            _tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            _style: &renderer::Style,
            layout: Layout<'_>,
            _cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            {
                let style = Catalog::style(theme, self.class);
                let bounds = layout.bounds();

                let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
                let option_height = f32::from(self.text_line_height.to_absolute(text_size)) + self.padding.y();

                let offset = viewport.y - bounds.y;
                let start = (offset / option_height) as usize;
                let end = ((offset + viewport.height) / option_height).ceil() as usize;

                let visible_options = &self.options[start..end.min(self.options.len())];

                for (i, option) in visible_options.iter().enumerate() {
                    let i = start + i;
                    let is_selected = self.selected.contains(option);
                    let is_hovered = *self.hovered_option == Some(i);

                    let option_bounds = Rectangle {
                        x: bounds.x,
                        y: bounds.y + (option_height * i as f32),
                        width: bounds.width,
                        height: option_height,
                    };

                    let box_size = option_height * 0.6;
                    let box_bounds = Rectangle {
                        x: bounds.x + 5.0,
                        y: bounds.y + 5.0 + (option_height * i as f32),
                        width: box_size,
                        height: box_size,
                    };

                    if is_hovered {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: option_bounds.x + style.border.width,
                                    width: option_bounds.width - style.border.width * 2.0,
                                    ..option_bounds
                                },
                                border: border::rounded(style.border.radius),
                                ..renderer::Quad::default()
                            },
                            style.selected_background,
                        );
                    }

                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: box_bounds,
                            border: style.checkbox.border,
                            ..renderer::Quad::default()
                        },
                        style.checkbox.background,
                    );

                    let Icon {
                        font,
                        code_point,
                        size,
                        line_height,
                        shaping,
                    } = &self.icon;
                    let size = size.unwrap_or(Pixels(box_bounds.height * 0.7));
                    if is_selected {
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
                            if is_hovered {
                                style.selected_text_color
                            } else {
                                style.checkbox.icon_color
                            },
                            *viewport,
                        );
                    }

                    renderer.fill_text(
                        Text {
                            content: option.to_string(),
                            bounds: Size::new(f32::INFINITY, option_bounds.height),
                            size: text_size,
                            line_height: self.text_line_height,
                            font: self.font.unwrap_or_else(|| renderer.default_font()),
                            align_x: text::Alignment::Default,
                            align_y: alignment::Vertical::Center,
                            shaping: self.text_shaping,
                            wrapping: text::Wrapping::default(),
                        },
                        Point::new(
                            option_bounds.x + self.padding.left + box_size + 5.0,
                            option_bounds.center_y(),
                        ),
                        style.text_color,
                        *viewport,
                    );
                }
            }
        }
    }

    impl<'a, 'b, T, Message, Theme, Renderer> From<List<'a, 'b, T, Message, Theme, Renderer>>
        for Element<'a, Message, Theme, Renderer>
    where
        T: ToString + Clone + PartialEq,
        Message: 'a,
        Theme: 'a + Catalog,
        Renderer: 'a + text::Renderer,
        'b: 'a,
    {
        fn from(list: List<'a, 'b, T, Message, Theme, Renderer>) -> Self {
            Element::new(list)
        }
    }

    /// The appearance of a [`Menu`].
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Style {
        /// The [`Background`] of the menu.
        pub background: Background,
        /// The [`Border`] of the menu.
        pub border: Border,
        /// The text [`Color`] of the menu.
        pub text_color: Color,
        /// The text [`Color`] of a selected option in the menu.
        pub selected_text_color: Color,
        /// The background [`Color`] of a selected option in the menu.
        pub selected_background: Background,
        /// The [`Shadow`] of the menu.
        pub shadow: Shadow,
        /// The style of the checkbox
        pub checkbox: CheckboxStyle,
    }

    /// The theme catalog of a [`Menu`].
    pub trait Catalog: scrollable::Catalog {
        /// The item class of the [`Catalog`].
        type Class<'a>;

        /// The default class produced by the [`Catalog`].
        fn default<'a>() -> <Self as Catalog>::Class<'a>;

        /// The default class for the scrollable of the [`Menu`].
        fn default_scrollable<'a>() -> <Self as scrollable::Catalog>::Class<'a> {
            <Self as scrollable::Catalog>::default()
        }

        /// The [`Style`] of a class with the given status.
        fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style;
    }

    /// A styling function for a [`Menu`].
    pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> StyleFn<'a, Self> {
            Box::new(default)
        }

        fn style(&self, class: &StyleFn<'_, Self>) -> Style {
            class(self)
        }
    }

    /// The default style of the list of a [`Menu`].
    pub fn default(theme: &Theme) -> Style {
        let palette = theme.extended_palette();

        let checkbox = CheckboxStyle {
            background: Color::TRANSPARENT.into(),
            icon_color: palette.primary.strong.color,
            border: Border {
                color: palette.background.strong.color,
                width: 1.0,
                radius: 2.0.into(),
            },
            text_color: None,
        };

        Style {
            background: palette.background.weak.color.into(),
            border: Border {
                width: 1.0,
                radius: 0.0.into(),
                color: palette.background.strong.color,
            },
            text_color: palette.background.weak.text,
            selected_text_color: palette.primary.strong.text,
            selected_background: palette.primary.strong.color.into(),
            shadow: Shadow::default(),
            checkbox,
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Icon<Font> {
        pub font: Font,
        pub code_point: char,
        pub size: Option<Pixels>,
        pub line_height: text::LineHeight,
        pub shaping: text::Shaping,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct CheckboxStyle {
        pub background: Background,
        pub icon_color: Color,
        pub border: Border,
        pub text_color: Option<Color>,
    }
}
