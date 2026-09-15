use syn::{Attribute, Item};

pub fn validate_rust_reproduction_code(code: &str) -> Result<(), String> {
    if code.trim().is_empty() {
        return Err("generated reproduction code is empty".to_owned());
    }

    let parsed = syn::parse_file(code)
        .map_err(|error| format!("generated reproduction is not valid Rust syntax: {error}"))?;

    let test_count = count_test_functions(&parsed.items);

    match test_count {
        0 => {
            return Err("generated reproduction does not contain a #[test] function".to_owned());
        }

        1 => {}

        count => {
            return Err(format!(
                "generated reproduction contains {count} test functions; exactly one is required"
            ));
        }
    }

    validate_no_extra_functions(&parsed.items)?;

    Ok(())
}

pub fn normalize_rust_reproduction_code(
    code: &str,
    expected_test_name: &str,
) -> Result<String, String> {
    if code.trim().is_empty() {
        return Err("generated reproduction code is empty".to_owned());
    }

    let mut parsed = syn::parse_file(code)
        .map_err(|error| format!("generated reproduction is not valid Rust syntax: {error}"))?;

    let test_count = count_test_functions(&parsed.items);

    // Already valid shape.
    if test_count == 1 {
        return Ok(prettyplease::unparse(&parsed));
    }

    if test_count != 0 {
        return Err(format!(
            "cannot normalize reproduction containing {test_count} test functions"
        ));
    }

    if parsed.items.len() != 1 {
        return Err(
            "generated reproduction without #[test] must contain exactly one function".to_owned(),
        );
    }

    let Some(Item::Fn(function)) = parsed.items.first_mut() else {
        return Err(
            "generated reproduction without #[test] must contain exactly one function".to_owned(),
        );
    };

    if function.sig.ident != expected_test_name {
        return Err(format!(
            "generated function `{}` does not match declared reproduction test `{expected_test_name}`",
            function.sig.ident
        ));
    }

    function.attrs.push(syn::parse_quote!(#[test]));

    let normalized = prettyplease::unparse(&parsed);

    // Run the normal strict validator after repair.
    validate_rust_reproduction_code(&normalized)?;

    Ok(normalized)
}

fn count_test_functions(items: &[Item]) -> usize {
    let mut count = 0;

    for item in items {
        match item {
            Item::Fn(function) => {
                if has_test_attribute(&function.attrs) {
                    count += 1;
                }
            }

            Item::Mod(module) => {
                if let Some((_, nested_items)) = &module.content {
                    count += count_test_functions(nested_items);
                }
            }

            _ => {}
        }
    }

    count
}

fn validate_no_extra_functions(items: &[Item]) -> Result<(), String> {
    for item in items {
        match item {
            Item::Fn(function) => {
                if !has_test_attribute(&function.attrs) {
                    return Err(format!(
                        "generated reproduction defines non-test function `{}`; reproduction tests must use existing repository functions instead of redefining them",
                        function.sig.ident
                    ));
                }
            }

            Item::Mod(module) => {
                if let Some((_, nested_items)) = &module.content {
                    validate_no_extra_functions(nested_items)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn rename_test_function(items: &mut [Item], execution_name: &str, renamed: &mut bool) {
    for item in items {
        match item {
            Item::Fn(function) => {
                if has_test_attribute(&function.attrs) {
                    function.sig.ident = syn::Ident::new(execution_name, function.sig.ident.span());
                    *renamed = true;
                    return;
                }
            }

            Item::Mod(module) => {
                if let Some((_, nested_items)) = &mut module.content {
                    rename_test_function(nested_items, execution_name, renamed);
                    if *renamed {
                        return;
                    }
                }
            }
            _ => {}
        }
    }
}

fn validate_execution_name(value: &str) -> Result<(), String> {
    let mut chars = value.chars();

    let Some(first) = chars.next() else {
        return Err("execution test name is empty".to_owned());
    };

    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err("execution test name is not a valid Rust identifier".to_owned());
    }

    if !chars.all(|character| character == '_' || character.is_ascii_alphanumeric()) {
        return Err("execution test name is not a valid Rust identifier".to_owned());
    }

    Ok(())
}

fn has_test_attribute(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("test"))
}

pub fn prepare_rust_reproduction_code(code: &str, execution_name: &str) -> Result<String, String> {
    validate_execution_name(execution_name)?;

    validate_rust_reproduction_code(code)?;

    let mut parsed = syn::parse_file(code)
        .map_err(|error| format!("generated reproduction is not valid Rust syntax: {error}"))?;

    let mut renamed = false;

    rename_test_function(&mut parsed.items, execution_name, &mut renamed);

    if !renamed {
        return Err("unable to rename generated test".to_owned());
    }

    Ok(prettyplease::unparse(&parsed))
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
    fn rejects_multiple_tests() {
        let code = r#"
            #[test]
            fn first() {}

            #[test]
            fn second() {}
            "#;

        let result = validate_rust_reproduction_code(code);

        assert!(result.is_err());

        let error = result.expect_err("multiple tests should fail validation");

        assert!(error.contains("exactly one is required"));
    }

    #[test]
    fn rejects_reimplemented_production_function() {
        let code = r#"
            fn divide(a: i32, b: i32) -> i32 {
                a / b
            }

            #[test]
            fn reproduces_failure() {
                assert_eq!(divide(10, 2), 10);
            }
            "#;

        let result = validate_rust_reproduction_code(code);

        assert!(result.is_err());

        let error = result.expect_err("reimplemented production function should be rejected");

        assert!(error.contains("non-test function `divide`"));
    }

    #[test]
    fn renames_generated_test() {
        let code = r#"
            #[test]
            fn original_name() {
                assert_eq!(2 + 2, 5);
            }
            "#;

        let prepared = prepare_rust_reproduction_code(code, "rootproof_reproduction_h1")
            .expect("prepare reproduction");

        assert!(prepared.contains("fn rootproof_reproduction_h1"));

        assert!(!prepared.contains("fn original_name"));
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

    #[test]
    fn rejects_invalid_execution_name() {
        let code = r#"
                #[test]
                fn example() {
                    assert_eq!(2 + 2, 5);
                }
                "#;

        let result = prepare_rust_reproduction_code(code, "123-invalid-name");

        assert!(result.is_err());
    }

    #[test]
    fn adds_missing_test_attribute_to_declared_test() {
        let code = r#"
            fn intentionally_failing_test() {
                assert_eq!(divide(10, 2), 10);
            }
            "#;

        let normalized = normalize_rust_reproduction_code(code, "intentionally_failing_test")
            .expect("normalize reproduction");

        assert!(normalized.contains("#[test]"));

        assert!(normalized.contains("fn intentionally_failing_test"));
    }

    #[test]
    fn does_not_normalize_wrong_function_name() {
        let code = r#"
            fn something_else() {
                assert_eq!(2 + 2, 5);
            }
            "#;

        let result = normalize_rust_reproduction_code(code, "expected_test");

        assert!(result.is_err());
    }

    #[test]
    fn does_not_normalize_multiple_functions() {
        let code = r#"
            fn divide(a: i32, b: i32) -> i32 {
                a / b
            }

            fn reproduction() {
                assert_eq!(divide(10, 2), 10);
            }
            "#;

        let result = normalize_rust_reproduction_code(code, "reproduction");

        assert!(result.is_err());
    }
}
