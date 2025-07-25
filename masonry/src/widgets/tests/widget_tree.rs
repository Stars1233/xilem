// Copyright 2024 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

use insta::assert_debug_snapshot;

use crate::core::{NewWidget, Widget as _};
use crate::testing::{TestHarness, widget_ids};
use crate::theme::default_property_set;
use crate::widgets::{Flex, Label};

#[test]
fn access_grandchild_widget() {
    let [id_label] = widget_ids();

    let widget = Flex::column()
        .with_child(
            Flex::row()
                .with_child(
                    Flex::row()
                        .with_child(NewWidget::new_with_id(Label::new("Old text"), id_label))
                        .with_auto_id(),
                )
                .with_auto_id(),
        )
        .with_flex_spacer(1.0);

    let mut harness = TestHarness::create(default_property_set(), widget);

    dbg!(harness.root_widget());
    harness.edit_widget(id_label, |mut label| {
        let mut label = label.downcast::<Label>();
        Label::set_text(&mut label, "New text");
    });

    assert_debug_snapshot!(harness.root_widget());
}
