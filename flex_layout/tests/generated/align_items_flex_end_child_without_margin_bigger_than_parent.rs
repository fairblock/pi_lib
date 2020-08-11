fn print(count: &mut usize, id: usize, layout: &layout::tree::LayoutR) {
    *count += 1;
    println!("result: {:?} {:?} {:?}", *count, id, layout);
}
#[test]
fn align_items_flex_end_child_without_margin_bigger_than_parent() {
    let mut layout_tree = layout::tree::LayoutTree::default();
    layout_tree.insert(
        1,
        0,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            position_type: layout::style::PositionType::Absolute,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(1920.0),
                height: layout::style::Dimension::Points(1024.0),
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        2,
        1,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            align_items: layout::style::AlignItems::Center,
            justify_content: layout::style::JustifyContent::Center,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(50f32),
                height: layout::style::Dimension::Points(50f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        3,
        2,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            align_items: layout::style::AlignItems::FlexEnd,
            ..Default::default()
        },
    );
    layout_tree.insert(
        4,
        3,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(70f32),
                height: layout::style::Dimension::Points(70f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.compute(print, &mut 0);
    let layout = layout_tree.get_layout(2).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 50f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 50f32);
    assert_eq!(layout.rect.start, 0f32);
    assert_eq!(layout.rect.top, 0f32);
    let layout = layout_tree.get_layout(3).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 70f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 70f32);
    assert_eq!(layout.rect.start, -10f32);
    assert_eq!(layout.rect.top, -10f32);
    let layout = layout_tree.get_layout(4).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 70f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 70f32);
    assert_eq!(layout.rect.start, 0f32);
    assert_eq!(layout.rect.top, 0f32);
}
