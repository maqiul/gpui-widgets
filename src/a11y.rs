use std::rc::Rc;

use gpui::{
    AccessibleAction, App, ClickEvent, ElementId, Orientation, Role, SharedString,
    StatefulInteractiveElement, Toggled, Window, accesskit,
};

#[derive(Clone, Debug)]
pub(crate) struct Semantics {
    role: Role,
    label: SharedString,
    id: SharedString,
    origin: SharedString,
    description: Option<SharedString>,
    selected: Option<bool>,
    expanded: Option<bool>,
    toggled: Option<Toggled>,
    value: Option<SharedString>,
    placeholder: Option<SharedString>,
    numeric_value: Option<f64>,
    min_numeric_value: Option<f64>,
    max_numeric_value: Option<f64>,
    numeric_value_step: Option<f64>,
    orientation: Option<Orientation>,
    level: Option<usize>,
    position_in_set: Option<usize>,
    size_of_set: Option<usize>,
    row_index: Option<usize>,
    column_index: Option<usize>,
    row_count: Option<usize>,
    column_count: Option<usize>,
    disabled: bool,
    read_only: bool,
    invalid: bool,
    focusable: bool,
}

impl Semantics {
    #[track_caller]
    pub(crate) fn new(role: Role, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        let location = core::panic::Location::caller();
        let origin: SharedString = format!(
            "{}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        )
        .into();
        Self {
            role,
            id: format!("{}:{}", origin, label).into(),
            label,
            origin,
            description: None,
            selected: None,
            expanded: None,
            toggled: None,
            value: None,
            placeholder: None,
            numeric_value: None,
            min_numeric_value: None,
            max_numeric_value: None,
            numeric_value_step: None,
            orientation: None,
            level: None,
            position_in_set: None,
            size_of_set: None,
            row_index: None,
            column_index: None,
            row_count: None,
            column_count: None,
            disabled: false,
            read_only: false,
            invalid: false,
            focusable: false,
        }
    }

    pub(crate) fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    pub(crate) fn element_id(&self) -> ElementId {
        ElementId::Name(format!("a11y:{}:{:?}", self.origin, self.id).into())
    }

    pub(crate) fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub(crate) fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    pub(crate) fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    pub(crate) fn toggled(mut self, toggled: Toggled) -> Self {
        self.toggled = Some(toggled);
        self
    }

    pub(crate) fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub(crate) fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub(crate) fn numeric(mut self, value: f64) -> Self {
        self.numeric_value = Some(value);
        self
    }

    pub(crate) fn numeric_range(mut self, min: f64, max: f64) -> Self {
        self.min_numeric_value = Some(min);
        self.max_numeric_value = Some(max);
        self
    }

    pub(crate) fn numeric_step(mut self, step: f64) -> Self {
        self.numeric_value_step = Some(step);
        self
    }

    pub(crate) fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    pub(crate) fn level(mut self, level: usize) -> Self {
        self.level = Some(level);
        self
    }

    pub(crate) fn position(mut self, position: usize, size: usize) -> Self {
        self.position_in_set = Some(position);
        self.size_of_set = Some(size);
        self
    }

    pub(crate) fn row(mut self, index: usize, count: usize) -> Self {
        self.row_index = Some(index);
        self.row_count = Some(count);
        self
    }

    pub(crate) fn column(mut self, index: usize, count: usize) -> Self {
        self.column_index = Some(index);
        self.column_count = Some(count);
        self
    }

    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(crate) fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub(crate) fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub(crate) fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub(crate) fn role(&self) -> Role {
        self.role
    }

    pub(crate) fn apply<T: StatefulInteractiveElement>(self, element: T) -> T {
        let mut element = element;
        let semantic_id = self.element_id();
        let element_id = element
            .interactivity()
            .element_id
            .clone()
            .map(|existing| ElementId::Name(format!("a11y:{semantic_id:?}:{existing:?}").into()))
            .unwrap_or(semantic_id);
        element.interactivity().element_id = Some(element_id);
        element = element
            .role(self.role)
            .aria_label(self.label.clone())
            .accessibility_id(self.id.clone());
        if let Some(description) = &self.description {
            element = element.aria_description(description.clone());
        }
        if let Some(selected) = self.selected {
            element = element.aria_selected(selected);
        }
        if let Some(expanded) = self.expanded {
            element = element.aria_expanded(expanded);
        }
        if let Some(toggled) = self.toggled {
            element = element.aria_toggled(toggled);
        }
        if let Some(value) = &self.value {
            element = element.aria_value(value.clone());
        }
        if let Some(placeholder) = &self.placeholder {
            element = element.aria_placeholder(placeholder.clone());
        }
        if let Some(value) = self.numeric_value {
            element = element.aria_numeric_value(value);
        }
        if let Some(min) = self.min_numeric_value {
            element = element.aria_min_numeric_value(min);
        }
        if let Some(max) = self.max_numeric_value {
            element = element.aria_max_numeric_value(max);
        }
        if let Some(step) = self.numeric_value_step {
            element = element.aria_numeric_value_step(step);
        }
        if let Some(orientation) = self.orientation {
            element = element.aria_orientation(orientation);
        }
        if let Some(level) = self.level {
            element = element.aria_level(level);
        }
        if let Some(position) = self.position_in_set {
            element = element.aria_position_in_set(position);
        }
        if let Some(size) = self.size_of_set {
            element = element.aria_size_of_set(size);
        }
        if let Some(index) = self.row_index {
            element = element.aria_row_index(index);
        }
        if let Some(count) = self.row_count {
            element = element.aria_row_count(count);
        }
        if let Some(index) = self.column_index {
            element = element.aria_column_index(index);
        }
        if let Some(count) = self.column_count {
            element = element.aria_column_count(count);
        }
        if self.disabled || self.read_only || self.invalid {
            let disabled = self.disabled;
            let read_only = self.read_only;
            let invalid = self.invalid;
            element = element.a11y_synthetic_children(move |builder| {
                let node = builder.parent_node();
                if disabled {
                    node.set_disabled();
                }
                if read_only {
                    node.set_read_only();
                }
                if invalid {
                    node.set_invalid(accesskit::Invalid::True);
                }
            });
        }
        if self.focusable {
            element = element.focusable().tab_stop(true);
        }
        element
    }

    pub(crate) fn write(&self, node: &mut accesskit::Node) {
        node.set_author_id(self.id.to_string());
        node.set_label(self.label.to_string());
        if let Some(description) = &self.description {
            node.set_description(description.to_string());
        }
        if let Some(selected) = self.selected {
            node.set_selected(selected);
        }
        if let Some(expanded) = self.expanded {
            node.set_expanded(expanded);
        }
        if let Some(toggled) = self.toggled {
            node.set_toggled(toggled);
        }
        if let Some(value) = &self.value {
            node.set_value(value.to_string());
        }
        if let Some(placeholder) = &self.placeholder {
            node.set_placeholder(placeholder.to_string());
        }
        if let Some(value) = self.numeric_value {
            node.set_numeric_value(value);
        }
        if let Some(min) = self.min_numeric_value {
            node.set_min_numeric_value(min);
        }
        if let Some(max) = self.max_numeric_value {
            node.set_max_numeric_value(max);
        }
        if let Some(step) = self.numeric_value_step {
            node.set_numeric_value_step(step);
        }
        if let Some(orientation) = self.orientation {
            node.set_orientation(orientation);
        }
        if let Some(level) = self.level {
            node.set_level(level);
        }
        if let Some(position) = self.position_in_set {
            node.set_position_in_set(position);
        }
        if let Some(size) = self.size_of_set {
            node.set_size_of_set(size);
        }
        if let Some(index) = self.row_index {
            node.set_row_index(index);
        }
        if let Some(count) = self.row_count {
            node.set_row_count(count);
        }
        if let Some(index) = self.column_index {
            node.set_column_index(index);
        }
        if let Some(count) = self.column_count {
            node.set_column_count(count);
        }
        if self.disabled {
            node.set_disabled();
        }
        if self.read_only {
            node.set_read_only();
        }
        if self.invalid {
            node.set_invalid(accesskit::Invalid::True);
        }
    }
}

pub(crate) fn click<T: StatefulInteractiveElement>(
    element: T,
    listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> T {
    let listener = Rc::new(listener);
    let mouse_listener = listener.clone();
    let a11y_listener = listener;
    element
        .on_click(move |event, window, cx| mouse_listener(event, window, cx))
        .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
            a11y_listener(&ClickEvent::default(), window, cx)
        })
}

pub(crate) fn click_shared<T: StatefulInteractiveElement>(
    element: T,
    listener: Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>,
) -> T {
    click(element, move |event, window, cx| {
        listener(event, window, cx)
    })
}

pub(crate) fn click_void<T: StatefulInteractiveElement>(
    element: T,
    listener: impl Fn(&mut Window, &mut App) + 'static,
) -> T {
    click(element, move |_, window, cx| listener(window, cx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantics_writes_role_state_and_value() {
        let semantics = Semantics::new(Role::Slider, "Volume")
            .id("volume")
            .numeric(0.4)
            .numeric_range(0.0, 1.0)
            .numeric_step(0.1)
            .orientation(Orientation::Horizontal)
            .disabled(true);
        let mut node = accesskit::Node::new(semantics.role());
        semantics.write(&mut node);

        assert_eq!(node.role(), Role::Slider);
        assert_eq!(node.label(), Some("Volume"));
        assert_eq!(node.author_id(), Some("volume"));
        assert_eq!(node.numeric_value(), Some(0.4));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(1.0));
        assert_eq!(node.numeric_value_step(), Some(0.1));
        assert_eq!(node.orientation(), Some(Orientation::Horizontal));
        assert!(node.is_disabled());
    }

    #[test]
    fn semantics_distinguishes_mixed_and_disabled_states() {
        let mut node = accesskit::Node::new(Role::CheckBox);
        Semantics::new(Role::CheckBox, "Select all")
            .toggled(Toggled::Mixed)
            .read_only(true)
            .invalid(true)
            .write(&mut node);

        assert_eq!(node.toggled(), Some(Toggled::Mixed));
        assert!(node.is_read_only());
        assert_eq!(node.invalid(), Some(accesskit::Invalid::True));
    }
}
