extern crate over;

use over::{obj::Obj, OverError};

// Test that parsing malformed .over files results in correct errors being returned.
#[allow(clippy::cognitive_complexity)]
#[test]
fn errors() {
    macro_rules! error_helper {
        ($filename:expr, $error:expr) => {{
            error_helper!($filename, $error, "")
        }};

        ($filename:expr, $error1:expr, $error2:expr) => {{
            let full_name = format!("tests/test_files/errors/{}", $filename);

            match Obj::from_file(&full_name) {
                Err(OverError::ParseError(s)) => {
                    if $error2 != "" {
                        if s != format!("{}: {}", full_name, $error1)
                            && s != format!("{}: {}", full_name, $error2)
                        {
                            panic!(
                                "Error in {}: {:?}.\nExpected: {:#?}\nAlternatively: {:#?}",
                                $filename, s, $error1, $error2
                            );
                        }
                    } else {
                        if s != format!("{}: {}", full_name, $error1) {
                            panic!("Error in {}: {:?}.\nExpected: {:#?}", $filename, s, $error1);
                        }
                    }
                }
                res => {
                    if $error2 != "" {
                        panic!(
                            "No error occurred in {}: {:?}.\nExpected: {:#?}\nAlternatively: {:#?}",
                            $filename, res, $error1, $error2
                        )
                    } else {
                        panic!(
                            "No error occurred in {}: {:#?}.\nExpected: {:#?}",
                            $filename, res, $error1
                        )
                    }
                }
            }
        }};
    }

    error_helper!(
        "any1.over",
        "Could not apply operator \'+\' on types Arr(Arr(Arr(Int))) and Arr(Arr(Arr(Str))) at \
         line 1, column 26"
    );
    error_helper!(
        "any2.over",
        "Expected Tup(Arr(Arr(Int)), Arr(Arr(Int))) at line 4, column 5; found Tup(Arr(Arr(Str)), \
         Arr(Arr(Str)))"
    );
    error_helper!(
        "any3.over",
        "Could not apply operator \'+\' on types Arr(Tup(Arr(Arr(Int)), Arr(Arr(Int)))) and \
         Arr(Tup(Arr(Arr(Str)), Arr(Arr(Str)))) at line 5, column 16"
    );
    error_helper!(
        "arr_types.over",
        "Expected Arr(Tup(Int, Int)) at line 2, column 37; found Arr(Tup(Int, Str))"
    );
    error_helper!(
        "bad_global.over",
        "Global \"@global\" at line 1, column 9 could not be found"
    );
    error_helper!("decimal.over", "Invalid numeric value at line 1, column 10");
    error_helper!(
        "deep.over",
        "Exceeded maximum recursion depth (64) at line 1, column 78"
    );
    error_helper!(
        "deep_include.over",
        "Exceeded maximum recursion depth (64) at line 10, column 12"
    );
    error_helper!(
        "dot1.over",
        "Invalid use of dot notation on value of type Bool at line 1, column 6; value must be an \
         Obj, Arr, or Tup."
    );
    error_helper!(
        "dot2.over",
        "Invalid character \' \' for value at line 2, column 11"
    );
    error_helper!(
        "dot3.over",
        "Variable \"none\" at line 1, column 6 could not be found"
    );
    error_helper!(
        "dot4.over",
        "Variable \"six\" at line 3, column 10 could not be found"
    );
    error_helper!("dot5.over", "Unexpected end at line 2");
    error_helper!(
        "dot_global.over",
        "Invalid character \'@\' for value at line 4, column 10"
    );
    error_helper!(
        "dot_huge.over",
        "Invalid index 348734701382471230498713241343 at line 2, column 10"
    );
    error_helper!(
        "dot_tup.over",
        "Tup index 3 out of bounds at line 2, col 10"
    );
    error_helper!(
        "dot_tup2.over",
        "Invalid character \'-\' for value at line 2, column 10"
    );
    error_helper!(
        "dup_global.over",
        "Duplicate global \"@global\" at line 2, column 1"
    );
    error_helper!(
        "dup_parents.over",
        "Duplicate field \"^\" at line 5, column 9"
    );
    error_helper!(
        "empty_field.over",
        "Invalid character \':\' for field at line 1, column 1"
    );
    error_helper!(
        "empty_number.over",
        "Invalid character \'\\n\' for value at line 1, column 7",
        "Invalid character \'\\r\' for value at line 1, column 7"
    );
    error_helper!(
        "escape.over",
        "Invalid escape character \'a\' following backslash at line 1, column 12. If you meant to \
         write a backslash, use \'\\\\\'"
    );
    error_helper!(
        "field_true.over",
        "Invalid field name \"true\" at line 1, column 1"
    );
    error_helper!(
        "field_obj.over",
        "Invalid field name \"Obj\" at line 1, column 1"
    );
    error_helper!(
        "fuzz1.over",
        "Invalid closing bracket ')' at line 20, column 1; expected \']\'"
    );
    error_helper!(
        "fuzz2.over",
        "Invalid closing bracket ')' at line 22, column 2; expected none"
    );
    error_helper!(
        "fuzz3.over",
        "Exceeded maximum recursion depth (64) at line 5, column 65"
    );
    error_helper!("fuzz4.over", "Duplicate field \"M\" at line 22, column 1");
    error_helper!(
        "fuzz5.over",
        "Invalid character \'(\' for value at line 27, column 4"
    );
    error_helper!(
        "fuzz6.over",
        "Expected Int at line 22, column 1; found Frac"
    );
    error_helper!(
        "fuzz7.over",
        "Invalid character \'\\n\' for field at line 8, column 0",
        "Invalid character \'\\r\' for field at line 7, column 4"
    );
    error_helper!(
        "fuzz8.over",
        "Invalid character \'\"\' for value at line 34, column 3"
    );
    error_helper!(
        "fuzz9.over",
        "Type mismatch: expected Obj, found Null at line 18, col 4"
    );
    error_helper!("fuzz10.over", "Unexpected end at line 1");
    error_helper!(
        "fuzz11.over",
        "Could not apply operator \'+\' on types Str and Int at line 14, column 5"
    );
    error_helper!("fuzz12.over", "Invalid numeric value at line 6, column 18");
    error_helper!(
        "fuzz13.over",
        "Variable \"g\" at line 20, column 1 could not be found"
    );
    error_helper!(
        "fuzz14.over",
        "Could not apply operator \'+\' on types Arr(Arr(Int)) and Arr(Arr(Arr(Int))) at line 8, \
         column 5"
    );
    error_helper!(
        "include1.over",
        "Invalid character \'\"\' for value at line 1, column 14"
    );
    error_helper!(
        "include2.over",
        "Expected Str at line 1, column 12; found Int"
    );
    error_helper!(
        "include3.over",
        "Invalid include path \"/\" at line 1, column 12"
    );
    error_helper!(
        "include4.over",
        "Variable \"Blah\" at line 1, column 8 could not be found"
    );
    error_helper!(
        "include5.over",
        "Invalid closing bracket \'S\' at line 1, column 17; expected \'>\'"
    );
    error_helper!(
        "include6.over",
        "Expected Str at line 1, column 15; found Obj"
    );
    error_helper!(
        "include7.over",
        "Invalid value of type \"Bool\" at line 1, column 11; must be either a Str value or one \
         of the tokens \"Obj\", \"Arr\", \"Tup\", or \"Str\""
    );
    error_helper!(
        "include_self.over",
        "Tried to cyclically include file \"include_self.over\" at line 1, column 11"
    );
    error_helper!(
        "op_arr.over",
        "Could not apply operator \'+\' on types Arr(Int) and Arr(Str) at line 1, column 13"
    );
    error_helper!(
        "op_arr_tup.over",
        "Could not apply operator \'+\' on types Arr(Any) and Tup() at line 1, column 11"
    );
    error_helper!(
        "op_end.over",
        "Invalid character \'\\n\' for value at line 3, column 9",
        "Invalid character \'\\r\' for value at line 3, column 9"
    );
    error_helper!(
        "op_error.over",
        "Could not apply operator \'+\' on types Str and Int at line 1, column 16"
    );
    error_helper!(
        "op_multiple.over",
        "Could not apply operator \'+\' on types Tup() and Frac at line 1, column 9"
    );
    error_helper!(
        "op_unary1.over",
        "Could not apply operator \'+\' on type Null at line 2, column 10"
    );
    error_helper!(
        "op_unary2.over",
        "Could not apply operator \'-\' on type Str at line 2, column 10"
    );
    error_helper!(
        "underscore.over",
        "Variable \"_444_444\" at line 1, column 9 could not be found"
    );
    error_helper!(
        "underscore_multiple.over",
        "Invalid character \'_\' for value at line 1, column 16"
    );
    error_helper!("unexpected_end1.over", "Unexpected end at line 2");
    error_helper!("unexpected_end2.over", "Unexpected end at line 3");
    error_helper!("value_amp.over", "Invalid value \"@\" at line 1, column 8");
}
