fn print(count: &mut usize, id: usize, layout: &layout::tree::LayoutR) {
    *count += 1;
    debug_println!("result: {:?} {:?} {:?}", *count, id, layout);
}
#[test]
fn wrap_reverse_row_align_content_stretch() {
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
            flex_wrap: layout::style::FlexWrap::WrapReverse,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(100f32),
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
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(30f32),
                height: layout::style::Dimension::Points(10f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        4,
        2,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(30f32),
                height: layout::style::Dimension::Points(20f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        5,
        2,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(30f32),
                height: layout::style::Dimension::Points(30f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        6,
        2,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(30f32),
                height: layout::style::Dimension::Points(40f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        7,
        2,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(30f32),
                height: layout::style::Dimension::Points(50f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.compute(print, &mut 0);
    let layout = layout_tree.get_layout(2).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 100f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 80f32);
    assert_eq!(layout.rect.start, 0f32);
    assert_eq!(layout.rect.top, 0f32);
    let layout = layout_tree.get_layout(3).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 30f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 10f32);
    assert_eq!(layout.rect.start, 0f32);
    assert_eq!(layout.rect.top, 70f32);
    let layout = layout_tree.get_layout(4).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 30f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 20f32);
    assert_eq!(layout.rect.start, 30f32);
    assert_eq!(layout.rect.top, 60f32);
    let layout = layout_tree.get_layout(5).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 30f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 30f32);
    assert_eq!(layout.rect.start, 60f32);
    assert_eq!(layout.rect.top, 50f32);
    let layout = layout_tree.get_layout(6).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 30f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 40f32);
    assert_eq!(layout.rect.start, 0f32);
    assert_eq!(layout.rect.top, 10f32);
    let layout = layout_tree.get_layout(7).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 30f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 50f32);
    assert_eq!(layout.rect.start, 30f32);
    assert_eq!(layout.rect.top, 0f32);
}
