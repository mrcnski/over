#![allow(clippy::cognitive_complexity)]

extern crate num_bigint;
extern crate num_rational;
extern crate num_traits;
#[macro_use]
extern crate over;

mod errors;

use num_traits::ToPrimitive;
use over::{
    obj::{Obj, Pair},
    types::Type,
    value::Value,
    OverResult, ReferenceType,
};
#[cfg(test)]
use pretty_assertions::assert_eq;

// Make comparisons with ints a bit more concise.
fn get_int(obj: &Obj, field: &str) -> i64 {
    obj.get_int(field).unwrap().to_i64().unwrap()
}

// Test parsing of empty file.
#[test]
fn empty() -> OverResult<()> {
    let obj = Obj::from_file("tests/test_files/empty.over")?;

    assert_eq!(obj.len(), 0);

    Ok(())
}

// Test reading basic Ints, Strs, Bools, and Null.
// Also test that whitespace and comments are correctly ignored.
#[test]
fn basic() -> OverResult<()> {
    let obj = Obj::from_file("tests/test_files/basic.over")?;

    assert_eq!(get_int(&obj, "_a1"), 1);
    assert_eq!(get_int(&obj, "a2"), 2);
    assert_eq!(get_int(&obj, "_"), 0);
    assert_eq!(obj.get("b").unwrap(), "Smörgåsbord");
    assert_eq!(get_int(&obj, "c"), 10);
    assert_eq!(get_int(&obj, "d"), 20);
    assert_eq!(get_int(&obj, "eee"), 2);
    assert_eq!(get_int(&obj, "f"), 3);
    assert_eq!(get_int(&obj, "g_"), 4);
    assert_eq!(obj.get("Hello").unwrap(), "Hello");
    assert_eq!(obj.get("i_robot").unwrap(), "not #a comment");
    assert_eq!(get_int(&obj, "j"), 4);
    assert_eq!(obj.get("k").unwrap(), "hi");
    assert_eq!(obj.get("l").unwrap(), "$\\\"");
    assert_eq!(obj.get("m").unwrap(), "m");
    assert_eq!(obj.get("n").unwrap(), true);
    assert_eq!(obj.get("o").unwrap(), false);
    assert_eq!(obj.get("p").unwrap(), "Hello");
    assert_eq!(get_int(&obj, "q"), 0);
    assert_eq!(obj.get("r").unwrap(), Value::Null);
    assert_eq!(obj.get("s").unwrap(), "\'");
    assert_eq!(obj.get("t").unwrap(), "\n");
    assert_eq!(obj.get("u").unwrap(), " ");
    assert_eq!(obj.get("v").unwrap(), "\'");
    assert_eq!(obj.get("w").unwrap(), "$");
    assert_eq!(obj.get_frac("x").unwrap(), frac!(1, 1));
    assert_eq!(obj.get("x").unwrap().get_frac().unwrap(), frac!(1, 1));

    Ok(())
}

// Test the example from the README.
#[test]
fn example() {
    let obj = Obj::from_file("tests/test_files/example.over").unwrap();

    assert_eq!(obj.get("receipt").unwrap(), "Oz-Ware Purchase Invoice");
    assert_eq!(obj.get("date").unwrap(), "2012-08-06");
    assert_eq!(
        obj.get("customer").unwrap(),
        obj! {
            "first_name" => "Dorothy",
            "family_name" => "Gale"
        }
    );

    assert_eq!(
        obj.get("items").unwrap(),
        arr![
            obj! {
                "part_no" => "A4786",
                "descrip" => "Water Bucket (Filled)",
                "price" => frac!(147,100),
                "quantity" => 4
            },
            obj! {
                "part_no" => "E1628",
                "descrip" => "High Heeled \"Ruby\" Slippers",
                "size" => 8,
                "price" => frac!(1337,10),
                "quantity" => 1
            },
        ]
    );

    assert!(
        obj.get("bill_to").unwrap()
            == obj! {
                "street" => "123 Tornado Alley\nSuite 16",
                "city" => "East Centerville",
                "state" => "KS",
            }
            || obj.get("bill_to").unwrap()
                == obj! {
                    "street" => "123 Tornado Alley\r\nSuite 16",
                    "city" => "East Centerville",
                    "state" => "KS",
                }
    );

    assert_eq!(obj.get("ship_to").unwrap(), obj.get("bill_to").unwrap());

    assert_eq!(
        obj.get("specialDelivery").unwrap(),
        "Follow the Yellow Brick Road to the Emerald City. Pay no attention to the man behind the \
         curtain."
    );
}

// Test parsing of sub-Objs.
#[test]
fn obj() {
    let obj = Obj::from_file("tests/test_files/obj.over").unwrap();

    assert_eq!(obj.get_obj("empty").unwrap().len(), 0);
    assert_eq!(obj.get_obj("empty2").unwrap().len(), 0);

    assert!(!obj.contains("bools"));
    let bools = obj! {"t" => true, "f" => false};
    let bools1 = obj.get_obj("bools1").unwrap();
    assert_eq!(bools1.num_references(), 5); // Include the reference created above.

    let outie = obj.get_obj("outie").unwrap();
    assert_eq!(outie.get_parent().unwrap(), bools);
    assert_eq!(get_int(&outie, "z"), 0);
    assert_eq!(outie.num_references(), 2); // Include the reference created above.

    let inner = outie.get_obj("inner").unwrap();
    assert_eq!(get_int(&inner, "z"), 1);
    let innie = inner.get_obj("innie").unwrap();
    assert_eq!(get_int(&innie, "a"), 1);
    assert_eq!(inner.get("b").unwrap(), tup!(1, 2,));

    assert_eq!(get_int(&outie, "c"), 3);
    assert_eq!(outie.get("d").unwrap(), obj! {});

    let obj_arr = obj.get_obj("obj_arr").unwrap();
    assert_eq!(obj_arr.get("arr").unwrap(), arr![1, 2, 3]);

    assert_eq!(obj.get_int("dot").unwrap(), int!(1));
    assert_eq!(obj.get_bool("dot_glob").unwrap(), true);
    assert_eq!(obj.get_int("dot_tup1").unwrap(), int!(1));
    assert_eq!(obj.get_int("dot_tup2").unwrap(), int!(2));
    assert_eq!(obj.get_int("dot_arr").unwrap(), int!(1));
    assert_eq!(obj.get_int("dot_op").unwrap(), int!(4));

    assert_eq!(obj.get_str("dot_var").unwrap(), "test");

    assert_eq!(obj.iter().count(), 15);
    let Pair(_, value) = obj.iter().last().unwrap();
    assert!(!value.is_null());
}

// Test that globals are referenced correctly and don't get included as fields.
#[test]
fn globals() -> OverResult<()> {
    let obj = Obj::from_file("tests/test_files/globals.over")?;

    let sub = obj.get_obj("sub")?;

    assert_eq!(sub.get_int("a")?, int!(1));
    assert_eq!(get_int(&sub, "b"), 2);
    assert_eq!(sub.len(), 2);

    assert_eq!(get_int(&obj, "c"), 2);

    assert_eq!(obj.len(), 3);

    Ok(())
}

// Test parsing of numbers.
#[test]
fn numbers() -> OverResult<()> {
    let obj = Obj::from_file("tests/test_files/numbers.over")?;

    assert_eq!(get_int(&obj, "neg"), -4);
    assert_eq!(obj.get_frac("pos")?, frac!(4, 1));
    assert_eq!(obj.get_frac("neg_zero")?, frac!(0, 1));
    assert_eq!(obj.get_frac("pos_zero")?, frac!(0, 1));

    assert_eq!(obj.get("frac_from_dec").unwrap(), frac!(13, 10));
    assert_eq!(obj.get("neg_ffd").unwrap(), frac!(-13, 10));
    assert_eq!(obj.get("pos_ffd").unwrap(), frac!(13, 10));

    assert_eq!(obj.get("add_dec").unwrap(), frac!(3, 1));
    assert_eq!(obj.get("sub_dec").unwrap(), frac!(-3, 1));

    let frac = obj.get_frac("big_frac").unwrap();
    assert!(frac > frac!(91_000_000, 1));
    assert!(frac < frac!(92_000_000, 1));

    assert_eq!(obj.get("frac1").unwrap(), frac!(1, 2));
    assert_eq!(obj.get("frac2").unwrap(), frac!(1, 2));
    assert_eq!(obj.get("frac3").unwrap(), frac!(0, 10));
    assert_eq!(obj.get("frac4").unwrap(), frac!(-5, 4));
    assert_eq!(obj.get("frac5").unwrap(), frac!(1, 1));

    assert_eq!(obj.get("whole_frac").unwrap(), frac!(3, 2));
    assert_eq!(obj.get("neg_whole_frac").unwrap(), frac!(-21, 4));
    assert_eq!(obj.get("dec_frac").unwrap(), frac!(1, 2));
    assert_eq!(obj.get("dec_frac2").unwrap(), frac!(-1, 2));

    assert_eq!(
        obj.get("array").unwrap(),
        arr![
            obj.get_frac("whole_frac").unwrap(),
            frac!(-1, 2),
            frac!(3, 2),
            frac!(1, 1),
        ]
    );

    assert_eq!(
        obj.get("tup").unwrap(),
        tup!(
            frac!(-1, 2),
            obj.get_frac("whole_frac").unwrap(),
            frac!(1, 1),
            frac!(3, 2),
        )
    );

    assert_eq!(obj.get("var_frac").unwrap(), frac!(-1, 2));

    Ok(())
}

#[test]
fn operations() -> OverResult<()> {
    let obj = Obj::from_file("tests/test_files/operations.over")?;

    assert_eq!(obj.get("mod1").unwrap(), int!(5));
    assert_eq!(obj.get("mod2").unwrap(), int!(0));

    assert_eq!(obj.get("arr1").unwrap(), arr![3, 4]);
    assert_eq!(obj.get("arr2").unwrap(), arr![3, 4]);
    assert_eq!(obj.get("arr3").unwrap(), arr![3, 4]);
    assert_eq!(obj.get("arr4").unwrap(), arr![arr![1]]);

    assert_eq!(
        obj.get("arr_complex").unwrap(),
        arr![arr![arr![1, 2]], arr![arr![3]]]
    );

    assert_eq!(obj.get("str1").unwrap(), "cat");
    assert_eq!(obj.get("str2").unwrap(), "cat");
    assert_eq!(obj.get("str3").unwrap(), "cat");
    assert_eq!(obj.get("str4").unwrap(), "cat");

    assert_eq!(
        obj.get("tup_complex").unwrap(),
        tup!(arr![arr![], arr![arr![], arr![1, 2, 3], arr![]]])
    );

    Ok(())
}

#[test]
fn any_type() -> OverResult<()> {
    let obj = Obj::from_file("tests/test_files/any_type.over")?;

    let arr1 = obj.get("arr1").unwrap();
    let arr2 = arr![arr![arr![]], arr![arr![2]], arr![arr![]]];
    assert_eq!(arr1.get_type(), Type::Arr(Box::new(arr2.inner_type())));
    assert_eq!(arr1, arr2);

    assert_eq!(
        obj.get("arr2").unwrap(),
        arr![
            tup!(arr![arr![]], arr![arr![2]]),
            tup!(arr![arr![2]], arr![arr![]]),
        ]
    );

    Ok(())
}

#[test]
fn includes() -> OverResult<()> {
    let obj = Obj::from_file("tests/test_files/includes.over").unwrap();

    // Test both \n and \r\n line endings.
    let s1 = "Multi-line string\nwhich should be included verbatim\nin another file. \"Quotes\" \
              and $$$\ndon't need to be escaped.\n";
    let s2 = "Multi-line string\r\nwhich should be included verbatim\r\nin another file. \
              \"Quotes\" and $$$\r\ndon't need to be escaped.\r\n";

    let include = obj.get("include").unwrap();
    assert!(include == s1 || include == s2);
    assert_eq!(obj.get("include2").unwrap(), obj.get("include").unwrap());

    assert_eq!(obj.get("include_arr").unwrap(), arr![1, 2, 3, 4, 5]);

    assert_eq!(
        obj.get("include_tup").unwrap(),
        tup!("hello", 1, "c", frac!(3, 3))
    );

    let o = obj.get_obj("include_obj").unwrap();
    assert_eq!(
        o,
        obj! {
            "obj2" => obj!{"test" => 1},
            "obj3" => obj!{"test" => 2},
            "dup" => obj!{"test" => 2},
        }
    );

    assert!(o.ptr_eq(&obj.get_obj("include_obj2")?));

    Ok(())
}

// TODO: Test multi-line.over (need substitution)

// Test writing objects to files.
#[test]
fn write() -> OverResult<()> {
    let write_dir = "tests/test_files/tmp";

    // Create temporary directory.
    match std::fs::create_dir(write_dir) {
        Ok(_) => (),
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        _ => panic!("Error creating directory"),
    }

    let write_path = format!("{}/write.over", write_dir);
    let write_path = write_path.as_str();

    macro_rules! write_helper {
        ($filename:expr) => {{
            let obj1 = Obj::from_file($filename)?;
            obj1.write_to_file(write_path)?;

            let obj2 = Obj::from_file(write_path)?;
            assert_eq!(obj1, obj2);
        }};
    }

    write_helper!("tests/test_files/basic.over");
    write_helper!("tests/test_files/empty.over");
    write_helper!("tests/test_files/example.over");
    write_helper!("tests/test_files/fuzz1.over");
    write_helper!("tests/test_files/fuzz2.over");
    write_helper!("tests/test_files/fuzz3.over");
    write_helper!("tests/test_files/fuzz4.over");
    write_helper!("tests/test_files/includes.over");
    write_helper!("tests/test_files/numbers.over");

    let obj1 = Obj::from_file("tests/test_files/obj.over")?;
    obj1.write_to_file(write_path)?;
    let obj2 = Obj::from_file(write_path)?;
    assert_eq!(obj1, obj2);
    // assert_eq!(obj2.get_obj("bools1")?.id(), obj2.get_obj("bools2")?.id());

    std::fs::remove_dir_all(write_dir).unwrap();

    Ok(())
}

// Test that unicode content survives the byte-level scanning paths in the parser
// (multibyte field names, string contents spanning lines, and values after them).
#[test]
fn unicode() -> OverResult<()> {
    let src = "h\u{e9}llo_w\u{f6}rld: \"\u{591a}\u{5b57}\u{8282} \u{1f680} line one\nline tw\u{f6}\"\nna\u{ef}ve: 1_000\nafter: \"ok\\n\"";
    let obj: Obj = src.parse()?;

    assert_eq!(
        obj.get("h\u{e9}llo_w\u{f6}rld").unwrap(),
        "\u{591a}\u{5b57}\u{8282} \u{1f680} line one\nline tw\u{f6}"
    );
    assert_eq!(obj.get("na\u{ef}ve").unwrap(), 1000);
    assert_eq!(obj.get("after").unwrap(), "ok\n");

    Ok(())
}
