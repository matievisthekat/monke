use crate::values::Value;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Default)]
pub(crate) struct Env<'parent> {
  bindings: HashMap<String, Value>,
  parent: Option<&'parent Self>,
}

impl<'parent> Env<'parent> {
  pub(crate) fn store_binding(&mut self, name: String, val: Value) {
    self.bindings.insert(name, val);
  }

  pub(crate) fn get_binding_value(&self, name: &str) -> Result<Value, String> {
    self
      .get_binding_value_without_error_msg(name)
      .ok_or_else(|| format!("binding with name ‘{}’ does not exist", name))
  }

  fn get_binding_value_without_error_msg(&self, name: &str) -> Option<Value> {
    self.bindings.get(name).cloned().or_else(|| {
      self
        .parent
        .and_then(|parent| parent.get_binding_value_without_error_msg(name))
    })
  }

  pub(crate) fn create_child(&'parent self) -> Self {
    Self {
      bindings: HashMap::new(),
      parent: Some(self),
    }
  }
}
