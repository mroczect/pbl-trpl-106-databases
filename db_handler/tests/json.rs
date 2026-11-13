#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use std::collections::{BTreeMap, HashMap};

use db_handler::json::{
    empty_object, expect_object, from_json, require_field, to_json, Json, Map, Number,
};
use db_handler::types::core::PageRequest;
use db_handler::DbError;
use serde::{Deserialize, Serialize};
use serde_json::json;


#[test]
fn empty_object_is_empty_map() {
    let j = empty_object();
    let obj = j.as_object().unwrap();
    assert!(obj.is_empty());
}

#[test]
fn empty_object_is_not_null() {
    assert!(!empty_object().is_null());
}

#[test]
fn empty_object_is_not_array() {
    assert!(!empty_object().is_array());
}

#[test]
fn empty_object_is_not_string() {
    assert!(!empty_object().is_string());
}

#[test]
fn empty_object_is_not_number() {
    assert!(!empty_object().is_number());
}

#[test]
fn empty_object_is_not_bool() {
    assert!(!empty_object().is_boolean());
}

#[test]
fn empty_object_serializes_to_braces() {
    assert_eq!(empty_object().to_string(), "{}");
}

#[test]
fn empty_object_calls_are_equal_but_independent() {
    let a = empty_object();
    let b = empty_object();
    assert_eq!(a, b);
}

#[test]
fn empty_object_can_be_mutated_after_creation() {
    let mut j = empty_object();
    let previous = j.as_object_mut().unwrap().insert("k".into(), json!(1));
    assert!(previous.is_none(), "insert into empty object must return None");
    assert_eq!(j, json!({"k": 1}));
    assert_eq!(empty_object(), json!({}));
}


#[test]
fn to_json_signed_integers() {
    assert_eq!(to_json(&0i8).unwrap(), json!(0));
    assert_eq!(to_json(&i8::MIN).unwrap(), json!(i8::MIN));
    assert_eq!(to_json(&i8::MAX).unwrap(), json!(i8::MAX));

    assert_eq!(to_json(&i16::MIN).unwrap(), json!(i16::MIN));
    assert_eq!(to_json(&i16::MAX).unwrap(), json!(i16::MAX));

    assert_eq!(to_json(&i32::MIN).unwrap(), json!(i32::MIN));
    assert_eq!(to_json(&i32::MAX).unwrap(), json!(i32::MAX));

    assert_eq!(to_json(&i64::MIN).unwrap(), json!(i64::MIN));
    assert_eq!(to_json(&i64::MAX).unwrap(), json!(i64::MAX));
}

#[test]
fn to_json_unsigned_integers() {
    assert_eq!(to_json(&0u8).unwrap(), json!(0));
    assert_eq!(to_json(&u8::MAX).unwrap(), json!(u8::MAX));
    assert_eq!(to_json(&u16::MAX).unwrap(), json!(u16::MAX));
    assert_eq!(to_json(&u32::MAX).unwrap(), json!(u32::MAX));
    assert_eq!(to_json(&u64::MAX).unwrap(), json!(u64::MAX));
}

#[test]
fn to_json_floats() {
    assert_eq!(to_json(&0.0f64).unwrap(), json!(0.0));
    assert_eq!(to_json(&1.5f64).unwrap(), json!(1.5));
    assert_eq!(to_json(&-1.5f64).unwrap(), json!(-1.5));
    assert_eq!(to_json(&f64::MAX).unwrap(), json!(f64::MAX));
    assert_eq!(to_json(&f64::MIN).unwrap(), json!(f64::MIN));
    assert_eq!(to_json(&1.25f32).unwrap(), json!(1.25));
}

#[test]
fn to_json_negative_zero_preserves_sign() {
    let json = to_json(&-0.0f64).unwrap();
    let n = json.as_f64().unwrap();
    assert!(n == 0.0 && n.is_sign_negative());
}

#[test]
fn to_json_bool() {
    assert_eq!(to_json(&true).unwrap(), json!(true));
    assert_eq!(to_json(&false).unwrap(), json!(false));
}

#[test]
fn to_json_string_variants() {
    assert_eq!(to_json(&"").unwrap(), json!(""));
    assert_eq!(to_json(&"hello").unwrap(), json!("hello"));
    assert_eq!(to_json(&"日本語").unwrap(), json!("日本語"));
    assert_eq!(to_json(&"🔥").unwrap(), json!("🔥"));
}

#[test]
fn to_json_string_with_special_chars() {
    assert_eq!(to_json(&"a\"b").unwrap(), json!("a\"b"));
    assert_eq!(to_json(&"a\\b").unwrap(), json!("a\\b"));
    assert_eq!(to_json(&"a\nb").unwrap(), json!("a\nb"));
    assert_eq!(to_json(&"a\tb").unwrap(), json!("a\tb"));
    assert_eq!(to_json(&"a\0b").unwrap(), json!("a\0b"));
}

#[test]
fn to_json_char_is_string() {
    assert_eq!(to_json(&'a').unwrap(), json!("a"));
    assert_eq!(to_json(&'🔥').unwrap(), json!("🔥"));
}

#[test]
fn to_json_unit_is_null() {
    assert_eq!(to_json(&()).unwrap(), Json::Null);
}


#[test]
fn to_json_vec() {
    assert_eq!(to_json(&vec![1, 2, 3]).unwrap(), json!([1, 2, 3]));
    assert_eq!(to_json(&Vec::<i32>::new()).unwrap(), json!([]));
    assert_eq!(
        to_json(&vec!["a".to_string(), "b".to_string()]).unwrap(),
        json!(["a", "b"])
    );
}

#[test]
fn to_json_array() {
    let arr = [1, 2, 3];
    assert_eq!(to_json(&arr).unwrap(), json!([1, 2, 3]));
}

#[test]
fn to_json_option_some() {
    assert_eq!(to_json(&Some(42)).unwrap(), json!(42));
    assert_eq!(to_json(&Some("x".to_string())).unwrap(), json!("x"));
}

#[test]
fn to_json_option_none_is_null() {
    assert_eq!(to_json(&Option::<i32>::None).unwrap(), Json::Null);
    assert_eq!(to_json(&Option::<String>::None).unwrap(), Json::Null);
}

#[test]
fn to_json_tuple() {
    assert_eq!(to_json(&(1, "a", true)).unwrap(), json!([1, "a", true]));
}

#[test]
fn to_json_hashmap() {
    let mut m: HashMap<String, i32> = HashMap::new();
    let _ = m.insert("a".into(), 1);
    let _ = m.insert("b".into(), 2);
    assert_eq!(to_json(&m).unwrap(), json!({"a": 1, "b": 2}));
}

#[test]
fn to_json_btreemap() {
    let mut m: BTreeMap<String, i32> = BTreeMap::new();
    let _ = m.insert("a".into(), 1);
    let _ = m.insert("b".into(), 2);
    assert_eq!(to_json(&m).unwrap(), json!({"a": 1, "b": 2}));
}

#[test]
fn to_json_nested_structure() {
    let value = json!({"outer": {"inner": [1, 2, 3]}});
    assert_eq!(to_json(&value).unwrap(), value);
}


#[test]
fn to_json_page_request() {
    let req = PageRequest {
        page: 2,
        per_page: 30,
    };
    assert_eq!(to_json(&req).unwrap(), json!({"page": 2, "per_page": 30}));
}

#[derive(Serialize)]
struct Widget {
    id: u32,
    name: String,
    tags: Vec<String>,
}

#[test]
fn to_json_custom_struct() {
    let w = Widget {
        id: 7,
        name: "sprocket".into(),
        tags: vec!["metal".into(), "small".into()],
    };
    assert_eq!(
        to_json(&w).unwrap(),
        json!({"id": 7, "name": "sprocket", "tags": ["metal", "small"]})
    );
}

#[derive(Serialize)]
struct WithOptional {
    maybe: Option<i32>,
    present: Option<i32>,
}

#[test]
fn to_json_struct_none_field_serializes_to_null() {
    // No #[serde(skip_serializing_if)] on this struct, so None becomes null.
    let x = WithOptional {
        maybe: None,
        present: Some(9),
    };
    assert_eq!(to_json(&x).unwrap(), json!({"maybe": null, "present": 9}));
}


#[test]
fn to_json_nan_collapses_to_null() {
    assert_eq!(to_json(&f64::NAN).unwrap(), Json::Null);
    assert_eq!(to_json(&f32::NAN).unwrap(), Json::Null);
}

#[test]
fn to_json_positive_infinity_collapses_to_null() {
    assert_eq!(to_json(&f64::INFINITY).unwrap(), Json::Null);
    assert_eq!(to_json(&f32::INFINITY).unwrap(), Json::Null);
}

#[test]
fn to_json_negative_infinity_collapses_to_null() {
    assert_eq!(to_json(&f64::NEG_INFINITY).unwrap(), Json::Null);
    assert_eq!(to_json(&f32::NEG_INFINITY).unwrap(), Json::Null);
}

#[test]
fn to_json_non_finite_inside_struct_collapses_to_null() {
    #[derive(Serialize)]
    struct WithFloat {
        x: f64,
        label: &'static str,
    }
    let v = WithFloat {
        x: f64::NAN,
        label: "ok",
    };
    assert_eq!(to_json(&v).unwrap(), json!({"x": null, "label": "ok"}));
}

#[test]
fn to_string_errors_on_non_finite_contrast_with_to_value() {
    assert!(serde_json::to_string(&f64::NAN).is_err());
    assert!(serde_json::to_string(&f64::INFINITY).is_err());
    assert!(serde_json::to_string(&f64::NEG_INFINITY).is_err());
    assert_eq!(serde_json::to_value(f64::NAN).unwrap(), Json::Null);
}


#[test]
fn from_json_signed_integers() {
    assert_eq!(from_json::<i8>(&json!(0)).unwrap(), 0);
    assert_eq!(from_json::<i8>(&json!(i8::MIN)).unwrap(), i8::MIN);
    assert_eq!(from_json::<i8>(&json!(i8::MAX)).unwrap(), i8::MAX);
    assert_eq!(from_json::<i32>(&json!(i32::MIN)).unwrap(), i32::MIN);
    assert_eq!(from_json::<i64>(&json!(i64::MIN)).unwrap(), i64::MIN);
    assert_eq!(from_json::<i64>(&json!(i64::MAX)).unwrap(), i64::MAX);
}

#[test]
fn from_json_unsigned_integers() {
    assert_eq!(from_json::<u8>(&json!(0)).unwrap(), 0);
    assert_eq!(from_json::<u8>(&json!(u8::MAX)).unwrap(), u8::MAX);
    assert_eq!(from_json::<u64>(&json!(u64::MAX)).unwrap(), u64::MAX);
}

#[test]
fn from_json_bool() {
    assert!(from_json::<bool>(&json!(true)).unwrap());
    assert!(!from_json::<bool>(&json!(false)).unwrap());
}

#[test]
fn from_json_string() {
    assert_eq!(from_json::<String>(&json!("")).unwrap(), "");
    assert_eq!(from_json::<String>(&json!("hello")).unwrap(), "hello");
    assert_eq!(from_json::<String>(&json!("日本語")).unwrap(), "日本語");
}

#[test]
fn from_json_float() {
    assert_eq!(from_json::<f64>(&json!(1.5)).unwrap(), 1.5);
    assert_eq!(from_json::<f64>(&json!(0.0)).unwrap(), 0.0);
    assert_eq!(from_json::<f64>(&json!(-1.5)).unwrap(), -1.5);
}

#[test]
fn from_json_integer_to_float_widens() {
    assert_eq!(from_json::<f64>(&json!(42)).unwrap(), 42.0);
}

#[test]
fn from_json_null_to_unit() {
    from_json::<()>(&Json::Null).unwrap();
}


#[test]
fn from_json_vec() {
    let v: Vec<i32> = from_json(&json!([1, 2, 3])).unwrap();
    assert_eq!(v, vec![1, 2, 3]);

    let empty: Vec<i32> = from_json(&json!([])).unwrap();
    assert!(empty.is_empty());
}

#[test]
fn from_json_option_some() {
    let x: Option<i32> = from_json(&json!(42)).unwrap();
    assert_eq!(x, Some(42));
}

#[test]
fn from_json_option_null_is_none() {
    let x: Option<i32> = from_json(&json!(null)).unwrap();
    assert_eq!(x, None);
}

#[test]
fn from_json_tuple() {
    let t: (i32, String, bool) = from_json(&json!([1, "a", true])).unwrap();
    assert_eq!(t, (1, "a".to_string(), true));
}

#[test]
fn from_json_hashmap() {
    let m: HashMap<String, i32> = from_json(&json!({"a": 1, "b": 2})).unwrap();
    assert_eq!(m.get("a"), Some(&1));
    assert_eq!(m.get("b"), Some(&2));
}

#[test]
fn from_json_nested() {
    #[derive(Deserialize)]
    struct Outer {
        inner: Vec<i32>,
    }
    let o: Outer = from_json(&json!({"inner": [1, 2, 3]})).unwrap();
    assert_eq!(o.inner, vec![1, 2, 3]);
}


#[test]
fn from_json_page_request_from_empty_object_uses_defaults() {
    let req: PageRequest = from_json(&json!({})).unwrap();
    assert_eq!(req.page, 1);
    assert_eq!(req.per_page, db_handler::constants::DEFAULT_PAGE_SIZE);
}

#[test]
fn from_json_page_request_partial() {
    let req: PageRequest = from_json(&json!({"page": 5})).unwrap();
    assert_eq!(req.page, 5);
    assert_eq!(req.per_page, db_handler::constants::DEFAULT_PAGE_SIZE);
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn from_json_struct_with_all_fields() {
    let p: Point = from_json(&json!({"x": 3, "y": 4})).unwrap();
    assert_eq!(p, Point { x: 3, y: 4 });
}

#[test]
fn from_json_struct_ignores_extra_fields_by_default() {
    let p: Point = from_json(&json!({"x": 3, "y": 4, "z": 99})).unwrap();
    assert_eq!(p, Point { x: 3, y: 4 });
}

#[test]
fn from_json_struct_accepts_array_in_field_order() {
    let p: Point = from_json(&json!([3, 4])).unwrap();
    assert_eq!(p, Point { x: 3, y: 4 });
}


#[test]
fn from_json_rejects_string_where_number_expected() {
    let err = from_json::<i32>(&json!("not-a-number")).unwrap_err();
    assert!(matches!(err, DbError::Json { .. }));
}

#[test]
fn from_json_rejects_number_where_string_expected() {
    assert!(from_json::<String>(&json!(42)).is_err());
}

#[test]
fn from_json_rejects_null_where_value_expected() {
    assert!(from_json::<i32>(&json!(null)).is_err());
    assert!(from_json::<String>(&json!(null)).is_err());
    assert!(from_json::<Point>(&json!(null)).is_err());
}

#[test]
fn from_json_rejects_array_where_map_expected() {
    assert!(from_json::<HashMap<String, i32>>(&json!([1, 2])).is_err());
}

#[test]
fn from_json_rejects_object_where_scalar_expected() {
    assert!(from_json::<i32>(&json!({"x": 1})).is_err());
}

#[test]
fn from_json_rejects_out_of_range_integer() {
    assert!(from_json::<u8>(&json!(256)).is_err());
    assert!(from_json::<i8>(&json!(128)).is_err());
    assert!(from_json::<i8>(&json!(-129)).is_err());
}

#[test]
fn from_json_rejects_negative_where_unsigned_expected() {
    assert!(from_json::<u32>(&json!(-1)).is_err());
}

#[test]
fn from_json_rejects_float_where_integer_expected() {
    assert!(from_json::<i32>(&json!(1.5)).is_err());
}

#[test]
fn from_json_rejects_bool_where_number_expected() {
    assert!(from_json::<i32>(&json!(true)).is_err());
}

#[test]
fn from_json_rejects_missing_required_field() {
    let err = from_json::<Point>(&json!({"x": 3})).unwrap_err();
    if let DbError::Json { message } = err {
        assert!(message.contains("missing field"), "message: {message}");
        assert!(message.contains('y'));
    } else {
        unreachable!("expected DbError::Json");
    }
}

#[test]
fn from_json_error_always_carries_nonempty_message() {
    for bad in [
        json!("not-a-struct"),
        json!(null),
        json!(42),
        json!(true),
        json!({}),
        json!({"x": 3}),
        json!({"x": "wrong type", "y": 4}),
    ] {
        let err = from_json::<Point>(&bad).unwrap_err();
        if let DbError::Json { message } = err {
            assert!(!message.is_empty(), "empty message for input {bad}");
        } else {
            unreachable!("expected DbError::Json for input {bad}");
        }
    }
}


fn roundtrip<T>(value: &T) -> T
where
    T: Serialize + serde::de::DeserializeOwned,
{
    let json = to_json(value).unwrap();
    from_json(&json).unwrap()
}

#[test]
fn roundtrip_scalar_values() {
    assert_eq!(roundtrip(&0i32), 0);
    assert_eq!(roundtrip(&i64::MIN), i64::MIN);
    assert_eq!(roundtrip(&u64::MAX), u64::MAX);
    assert_eq!(roundtrip(&1.5f64), 1.5);
    assert_eq!(roundtrip(&"hello".to_string()), "hello".to_string());
    assert!(roundtrip(&true));
}

#[test]
fn roundtrip_compound_values() {
    assert_eq!(roundtrip(&vec![1, 2, 3]), vec![1, 2, 3]);
    assert_eq!(
        roundtrip(&(1, "x".to_string(), false)),
        (1, "x".to_string(), false)
    );
    assert_eq!(roundtrip(&Some(42)), Some(42));
    assert_eq!(roundtrip(&Option::<i32>::None), None);
}

#[test]
fn roundtrip_struct() {
    let p = Point { x: 3, y: 4 };
    assert_eq!(roundtrip(&p), p);
}

#[test]
fn roundtrip_json_value_is_identity() {
    let original = json!({"a": [1, 2, 3], "b": {"c": null}});
    assert_eq!(to_json(&original).unwrap(), original);
}

#[test]
fn roundtrip_preserves_negative_zero() {
    let back: f64 = roundtrip(&-0.0f64);
    assert!(back == 0.0 && back.is_sign_negative());
}


#[test]
fn expect_object_on_object_returns_map() {
    let j = json!({"a": 1, "b": 2});
    let map = expect_object(&j).unwrap();
    assert_eq!(map.len(), 2);
}

#[test]
fn expect_object_on_empty_object_returns_empty_map() {
    let j = json!({});
    let map = expect_object(&j).unwrap();
    assert!(map.is_empty());
}

#[test]
fn expect_object_with_unicode_keys() {
    let j = json!({"nama": "Budi", "umur": 30});
    let map = expect_object(&j).unwrap();
    assert!(map.contains_key("nama"));
    assert!(map.contains_key("umur"));
}

#[test]
fn expect_object_with_nested_values() {
    let j = json!({"a": {"b": 1}});
    let map = expect_object(&j).unwrap();
    let inner = map.get("a").unwrap();
    assert!(inner.is_object());
}

#[test]
fn expect_object_returns_reference_not_clone() {
    let j = json!({"a": 1});
    let map = expect_object(&j).unwrap();
    let inner = map.get("a").unwrap();
    let direct = j.get("a").unwrap();
    assert!(std::ptr::eq(inner, direct));
}


#[test]
fn expect_object_on_null_fails_with_validation() {
    let err = expect_object(&json!(null)).unwrap_err();
    assert!(matches!(err, DbError::Validation { .. }));
}

#[test]
fn expect_object_on_array_fails_with_validation() {
    let err = expect_object(&json!([1, 2])).unwrap_err();
    assert!(matches!(err, DbError::Validation { .. }));
}

#[test]
fn expect_object_on_string_fails_with_validation() {
    let err = expect_object(&json!("hello")).unwrap_err();
    assert!(matches!(err, DbError::Validation { .. }));
}

#[test]
fn expect_object_on_number_fails_with_validation() {
    let err = expect_object(&json!(42)).unwrap_err();
    assert!(matches!(err, DbError::Validation { .. }));
}

#[test]
fn expect_object_on_bool_fails_with_validation() {
    let err = expect_object(&json!(true)).unwrap_err();
    assert!(matches!(err, DbError::Validation { .. }));
}


#[test]
fn require_field_existing_returns_value() {
    let j = json!({"name": "Alice"});
    let map = expect_object(&j).unwrap();
    let value = require_field(map, "name").unwrap();
    assert_eq!(value.as_str(), Some("Alice"));
}

#[test]
fn require_field_null_value_is_present() {
    let j = json!({"email": null});
    let map = expect_object(&j).unwrap();
    let value = require_field(map, "email").unwrap();
    assert!(value.is_null());
}

#[test]
fn require_field_unicode_field_name() {
    let j = json!({"nama_pengguna": "Budi"});
    let map = expect_object(&j).unwrap();
    let value = require_field(map, "nama_pengguna").unwrap();
    assert_eq!(value.as_str(), Some("Budi"));
}

#[test]
fn require_field_with_nested_value() {
    let j = json!({"user": {"id": 7}});
    let map = expect_object(&j).unwrap();
    let user = require_field(map, "user").unwrap();
    let inner = require_field(user.as_object().unwrap(), "id").unwrap();
    assert_eq!(inner.as_i64(), Some(7));
}

#[test]
fn require_field_returns_reference_into_map() {
    let j = json!({"k": {"nested": 42}});
    let map = expect_object(&j).unwrap();
    let via_require = require_field(map, "k").unwrap();
    let via_get = map.get("k").unwrap();
    assert!(std::ptr::eq(via_require, via_get));
}


#[test]
fn require_field_missing_fails_with_validation() {
    let j = json!({"name": "Alice"});
    let map = expect_object(&j).unwrap();
    let err = require_field(map, "email").unwrap_err();
    assert!(matches!(err, DbError::Validation { .. }));
}

#[test]
fn require_field_missing_error_mentions_field_name() {
    let j = json!({});
    let map = expect_object(&j).unwrap();
    let err = require_field(map, "user_id").unwrap_err();
    if let DbError::Validation { message } = err {
        assert!(message.contains("user_id"), "message: {message}");
    } else {
        unreachable!("expected DbError::Validation");
    }
}

#[test]
fn require_field_case_sensitive() {
    let j = json!({"Name": "Alice"});
    let map = expect_object(&j).unwrap();
    assert!(require_field(map, "name").is_err());
    assert!(require_field(map, "Name").is_ok());
}

#[test]
fn require_field_empty_name_is_missing_unless_present() {
    let j = json!({"": "empty-key"});
    let map = expect_object(&j).unwrap();
    let v = require_field(map, "").unwrap();
    assert_eq!(v.as_str(), Some("empty-key"));

    let j2 = json!({"a": 1});
    let map2 = expect_object(&j2).unwrap();
    assert!(require_field(map2, "").is_err());
}


#[test]
fn json_alias_equals_serde_json_value() {
    let a: Json = json!(42);
    let b: serde_json::Value = json!(42);
    assert_eq!(a, b);
}

#[test]
fn map_is_serde_json_map() {
    let mut m: Map<String, Json> = Map::new();
    let previous = m.insert("k".into(), json!(1));
    assert!(previous.is_none(), "insert into fresh Map must return None");
    assert_eq!(m.len(), 1);
    assert_eq!(m.get("k"), Some(&json!(1)));
}

#[test]
fn number_from_f64_rejects_nan_and_infinity() {
    assert!(Number::from_f64(f64::NAN).is_none());
    assert!(Number::from_f64(f64::INFINITY).is_none());
    assert!(Number::from_f64(f64::NEG_INFINITY).is_none());
    assert!(Number::from_f64(1.5).is_some());
}

#[test]
fn number_from_integers() {
    assert_eq!(Number::from(42i32).as_i64(), Some(42));
    assert_eq!(Number::from(42u32).as_u64(), Some(42));
}


#[test]
fn to_json_never_panics_on_any_json_input() {
    let cases = [
        Json::Null,
        json!(true),
        json!(42),
        json!(1.5),
        json!("s"),
        json!([]),
        json!({}),
    ];
    for value in cases {
        assert!(to_json(&value).is_ok(), "failed on {value}");
    }
}

#[test]
fn from_json_never_panics_on_any_json_input() {
    let cases = [
        Json::Null,
        json!(true),
        json!(42),
        json!("s"),
        json!([]),
        json!({}),
    ];
    for value in cases {
        let result = from_json::<Option<i32>>(&value);
        let _ = result.is_ok() || result.is_err();
    }
}

#[test]
fn expect_object_then_require_field_chain() {
    let j = json!({"user_id": 42, "role": "admin"});
    let map = expect_object(&j).unwrap();
    let uid = require_field(map, "user_id").unwrap().as_i64().unwrap();
    let role = require_field(map, "role").unwrap().as_str().unwrap();
    assert_eq!(uid, 42);
    assert_eq!(role, "admin");
}

#[test]
fn multiple_helper_calls_do_not_interfere() {
    let j1 = json!({"a": 1});
    let j2 = json!({"b": 2});
    let m1 = expect_object(&j1).unwrap();
    let m2 = expect_object(&j2).unwrap();
    assert!(m1.contains_key("a"));
    assert!(!m1.contains_key("b"));
    assert!(m2.contains_key("b"));
    assert!(!m2.contains_key("a"));
}
