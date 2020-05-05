/// The opaque component type
type ComponentImpl = ();

#[repr(C)]
struct ComponentType {
    /// Allocate an instance of this component
    create: fn(*const ComponentType) -> *mut ComponentImpl,

    /// destruct this component
    destroy: fn(*const ComponentType, *mut ComponentImpl),

    /// Returns an array that represent the item tree
    /// FIXME: dynamic items
    item_tree: fn(*const ComponentType) -> *const ItemTreeNode,
}

/// From the ItemTreeNode and a ComponentImpl, you can get a pointer to the instance data
/// ItemImpl via the offset field.
type ItemImpl = ();
// Example memory representation:
// offset| type | value
// 0     | f32 | x
// 4     | f32 | y
// ...
// 64    | RenderNode | render node index

#[repr(C)]
struct RenderNode {
    /// Used and modified by the backend, should be initialized to 0 by the user code
    cache_index: Cell<usize>,
    /// Set to true by the user code, and reset to false by the backend
    dirty_bit: Cell<bool>,
}

#[repr(C)]
struct ItemTreeNode {
    /// byte offset where we can find the item (from the *ComponentImpl)
    offset: isize,
    /// virtual table of the item
    vtable: *const ItemVTable,

    /// number of children
    chilren_count: u32,

    /// index of the first children
    children_index: u32,
}

#[repr(C)]
struct ItemVTable {

    // Rectangle: x/y/width/height ==> (path -> vertices/indicies(triangle))
    geometry: fn(*const ItemImpl) -> PrimitiveRectangle, // like kurbo::Rect

    // offset in bytes fromthe *const ItemImpl
    renderNodeIndexOffset: usize,
    // fn(*const ItemImpl) -> usize,

    // ???
    rendering_info: Option<fn(*const ItemImpl) -> RenderingInfo>,

    /// We would need max/min/preferred size, and all layout info
    layouting_info: Option<fn(*const ItemImpl) -> LayoutInfo>,

    /// input event
    input_event: Option<fn(*const ItemImpl, MouseEvent)>,
}

// given an ItemImpl & ItemVTable
// (1) Identify that the item *is* a rectangle or has everything a rectangle would have
// (2) change the width

#[repr(C)]
struct LayoutInfo {
    min_size: f32,
    //...
    width_offset: isize,
}

#[repr(C)]
enum RenderingInfo {
    NoContents,
    Path(Vec<PathElement>),
    Image(OpaqueImageHandle, AspectRatio),
    Text(String)
}


/*
Button { visible: false; text: "foo"}

 -> ProxyWithVisibility<NativeItem>

// Qt style selected:
fn render_button(item: *const ItemImpl) -> RenderingInfo {
    let button = reinterpret_cast<&Button>(button)
    let text = b.text();
    let isPressed = b.isPressed();
    // ...
    let image = qt_render_button(width, height, isPressed, text)

    return RenderingInfo::Image(image)
}

// Basic style selected:

 -> Rectangle / Text

 fn render_rectangle(item: *const ItemImpl) -> RenderingInfo {
     let rect = reinterpret_cast<&Rectangle>(item)
     ...
     return RenderingInfo::Path(rect_path)
 }

 fn render_text(item: *const ItemImpl) -> RenderInfo {

 }

*/

// in corelib/primitives.rs

bitflags! {
    enum ItemExtensions {
        HasVisibility,
        HasOpacity
    }
}

struct ItemBase {
extensions: ItemExtensions,
x: Property<f32>,
y: Property<f32>,
// visible, opacity, ?
extraData: Vec<...>
}

impl ItemBase {
    pub fn is_visible(&self) -> bool {
        if self.extensions & HasVisibility {
            return self.extraData
        } else {
            return true;
        }
    }
}

#[derive(SixtyFpsItem)]
/// ```
/// width: f32
/// height: f32
/// ```
fn render_rectangle(item: *const ItemImpl) -> RenderingInfo {
    //let width = property_at_offset(item, 1);
    let rect : &Rectnalge = unsafe { std::mem::transmute(item) };
    let width = rect.width.get()
}

//#[derive(SixtyFpsItem)]
struct Rectangle {
    base: ItemBase,
    width: Property<f32>,
    height: Property<f32>,
    radius: Property<f32>,
}

/// fn rect_from_item(item: *const ItemImpl) -> *const Rectangle {
///  reinterpet_cast<const Rectangle *>(reinterpret_cast<char *>(item) + ... some offset))
///}

pub static RECTANGLE_VTABLE: ItemVTable = ItemVTable {
    rendering_info: render_rectangle,
}

// in styles/qt.rs

//#[derive(SixtyFpsItem)]
struct QtButton {
    text: String,
    is_pressed: bool, 
}


pub static QT_BUTTON_VTABLE: ItemVTable = ItemVTable {
    rendering_info: render_qt_button,
};

// Button { text: "hello "}
const GENERATED : &str = r#"
#include <qtstyle.h>
/*
// generated with cbindgen
struct QtButton { String text; bool is_pressed };
*/

struct MyGeneratedComponent {
    QtButton button;

    MyGeneratedComponent() {
        button.text = "hello"
    }
};

ItemTreeNode array[] = {
    { 0, &QT_BUTTON_VTABLE,  0, 0 }
}

main() {
    MyGeneratedComponent foo;
sixtyfps_runtime::run(&foo, ComponentType{ nullptr, nullptr, []{return array}  })
}

"#