use std::rc::Rc;

use crate::ui::window::{MainWindow, WidgetItem, WidgetItemValue, WidgetKind};
use slint::{Image, ModelRc, SharedString, VecModel, Weak};

pub fn start_dynamic_service(window: Weak<MainWindow>) {
    let model = Rc::new(VecModel::<WidgetItem>::from(Vec::new()));
    let item = WidgetItem {
        title: SharedString::from("Hello"),
        value: WidgetItemValue {
            kind: WidgetKind::Text,
            value_string: SharedString::from("Hello"),
            value_image: Image::default(),
        },
    };
    model.push(item);
    let item2 = WidgetItem {
        title: SharedString::from("Hello2"),
        value: WidgetItemValue {
            kind: WidgetKind::Image,
            value_string: SharedString::from(""),
            value_image: Image::default(),
        },
    };
    model.push(item2);
    window
        .upgrade()
        .unwrap()
        .set_dashboard_items(ModelRc::from(model));
}
