//! The width of a single-line column flex container that is only asked for its width (here as an item of a
//! row whose flex base sizes are being determined), which is found without measuring the height of its items.
#[cfg(test)]
mod column_width {
    use taffy::prelude::*;

    /// A leaf that is 10 wide and 10 high.
    fn square(known_dimensions: Size<Option<f32>>, _available_space: Size<AvailableSpace>) -> Size<f32> {
        known_dimensions.unwrap_or(Size { width: 10.0, height: 10.0 })
    }

    /// A leaf as wide as the height it is given: its known height, else the height available to it, else 10.
    fn as_wide_as_high(known_dimensions: Size<Option<f32>>, available_space: Size<AvailableSpace>) -> Size<f32> {
        let height = known_dimensions.height.or(available_space.height.into_option());
        Size {
            width: known_dimensions.width.or(height).unwrap_or(10.0),
            height: known_dimensions.height.unwrap_or(10.0),
        }
    }

    /// The width of a column with `style` and `children`, as an item of a row in `available_height`, with the
    /// leaves measured by `measure`.
    fn column_width(
        taffy: &mut TaffyTree<()>,
        style: Style,
        children: &[NodeId],
        available_height: AvailableSpace,
        measure: fn(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>,
    ) -> f32 {
        let column =
            taffy.new_with_children(Style { flex_direction: FlexDirection::Column, ..style }, children).unwrap();
        let row = taffy.new_with_children(Style::default(), &[column]).unwrap();
        let available_space = Size { width: AvailableSpace::MaxContent, height: available_height };
        taffy
            .compute_layout_with_measure(row, available_space, |inputs, _, _, style| {
                taffy::compute_leaf_layout(inputs, style, |_, _| 0.0, measure)
            })
            .unwrap();
        taffy.layout(column).unwrap().size.width
    }

    fn fixed(width: f32, height: f32) -> Style {
        Style { size: Size { width: length(width), height: length(height) }, ..Default::default() }
    }

    fn height(height: f32) -> Style {
        Style { size: Size { width: auto(), height: length(height) }, ..Default::default() }
    }

    #[test]
    fn widest_item() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let narrow = taffy.new_leaf(fixed(30.0, 20.0)).unwrap();
        let wide =
            taffy.new_leaf(Style { margin: Rect { left: length(5.0), ..Rect::zero() }, ..fixed(50.0, 20.0) }).unwrap();
        let style = Style { padding: Rect { left: length(3.0), ..Rect::zero() }, ..Default::default() };
        assert_eq!(column_width(&mut taffy, style, &[narrow, wide], AvailableSpace::MaxContent, square), 58.0);
    }

    /// An item with an aspect ratio is as wide as the height it flexes to makes it.
    #[test]
    fn item_with_aspect_ratio() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let content = taffy.new_leaf(Style::default()).unwrap();
        let style = Style { display: Display::Block, flex_grow: 1.0, aspect_ratio: Some(2.0), ..Default::default() };
        let item = taffy.new_with_children(style, &[content]).unwrap();
        assert_eq!(column_width(&mut taffy, height(100.0), &[item], AvailableSpace::MaxContent, square), 200.0);
    }

    /// A column that wraps is as wide as its lines together.
    #[test]
    fn wrapping_column() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let items = [taffy.new_leaf(fixed(50.0, 60.0)).unwrap(), taffy.new_leaf(fixed(50.0, 60.0)).unwrap()];
        let style = Style { flex_wrap: FlexWrap::Wrap, ..height(100.0) };
        assert_eq!(column_width(&mut taffy, style, &items, AvailableSpace::MaxContent, square), 100.0);
    }

    /// An item is measured at the height its style gives it, within its minimum and maximum heights and no less
    /// than its padding and border.
    #[test]
    fn item_with_height() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let item = taffy.new_leaf(height(40.0)).unwrap();
        let width = column_width(&mut taffy, Style::default(), &[item], AvailableSpace::MaxContent, as_wide_as_high);
        assert_eq!(width, 40.0);
        let item =
            taffy.new_leaf(Style { max_size: Size { width: auto(), height: length(25.0) }, ..height(40.0) }).unwrap();
        let width = column_width(&mut taffy, Style::default(), &[item], AvailableSpace::MaxContent, as_wide_as_high);
        assert_eq!(width, 25.0);
        let item =
            taffy.new_leaf(Style { padding: Rect { top: length(20.0), ..Rect::zero() }, ..height(5.0) }).unwrap();
        let width = column_width(&mut taffy, Style::default(), &[item], AvailableSpace::MaxContent, as_wide_as_high);
        assert_eq!(width, 20.0);
    }

    /// An item with no height of its own is measured with its height unknown: in the height of the column,
    /// less the item's margins, if that is known, and if not in a column that will be as high as its items.
    #[test]
    fn item_available_height() {
        let margin = || Style { margin: Rect { top: length(30.0), ..Rect::zero() }, ..Default::default() };
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let item = taffy.new_leaf(margin()).unwrap();
        let width = column_width(&mut taffy, height(100.0), &[item], AvailableSpace::MaxContent, as_wide_as_high);
        assert_eq!(width, 70.0);
        let item = taffy.new_leaf(margin()).unwrap();
        let width = column_width(&mut taffy, Style::default(), &[item], AvailableSpace::MaxContent, as_wide_as_high);
        assert_eq!(width, 10.0);
    }
}
