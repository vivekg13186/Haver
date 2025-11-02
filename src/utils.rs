use rhai::Scope;

/// Sets a string variable in the Rhai scope.
pub fn set_string_var(name: &str, value: &str, scope: &mut Scope) {
    scope.set_value(name, value.to_string());
}
