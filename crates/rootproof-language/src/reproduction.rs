use syn::{Attribute, Item};

pub fn validate_rust_reproduction_code(code: &str) -> Result<(), String> {
    if code.trim().is_empty() {
        return Err("generated reproduction code is empty".to_owned());
    }

    let parsed = syn::parse_file(code)
        .map_err(|error| format!("generated reproduction is not valid Rust syntax: {error}"))?;

    if !contains_test_function(&parsed.items) {
        return Err("generated reproduction does not contain a #[test] function".to_owned());
    }

    Ok(())
}

fn contains_test_function(items: &[Item]) -> bool {
    for item in items {
        match item {
            Item::Fn(function) => {
                if has_test_attribute(&function.attrs) {
                    return true;
                }
            }

            Item::Mod(module) => {
                let Some((_brace, module_items)) = &module.content else {
                    continue;
                };

                if contains_test_function(module_items) {
                    return true;
                }
            }

            _ => {}
        }
    }

    false
}

fn has_test_attribute(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("test"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_top_level_test_function() {
        let code = r#"
            #[test]
            fn reproduces_failure() {
                assert_eq!(2 + 2, 5);
            }
            "#;
        let result = validate_rust_reproduction_code(code);

        assert!(result.is_ok());
    }

    #[test]
    fn accepts_test_inside_module() {
        let code = r#"
            mod tests {
                use super::*;

                #[test]
                fn reproduces_failure() {
                    assert_eq!(2 + 2, 5);
                }
            }
            "#;

        let result = validate_rust_reproduction_code(code);
        assert!(result.is_ok());
    }

    #[test]
    fn accepts_test_inside_nested_modules() {
        let code = r#"
            mod outer {
                mod inner {
                    #[test]
                    fn reproduces_failure() {
                        assert_eq!(2 + 2, 5);
                    }
                }
            }
            "#;

        let result = validate_rust_reproduction_code(code);

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_invalid_rust() {
        let code = r#"
            #[test]
            fn reproduces_failure( {
            "#;

        let result = validate_rust_reproduction_code(code);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_code_without_test_function() {
        let code = r#"
            fn helper() {
                println!("hello");
            }
            "#;

        let result = validate_rust_reproduction_code(code);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_module_without_test_function() {
        let code = r#"
            mod tests {
                fn helper() {
                    println!("hello");
                }
            }
            "#;

        let result = validate_rust_reproduction_code(code);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_empty_code() {
        let result = validate_rust_reproduction_code("");

        assert!(result.is_err());
    }
}
