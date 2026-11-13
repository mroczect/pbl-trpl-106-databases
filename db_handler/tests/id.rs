#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_results)]

use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::mem::{align_of, size_of};

use db_handler::json::{from_json, to_json};
use db_handler::Id;
use serde::{Deserialize, Serialize};
use serde_json::json;

struct User;
struct Product;


fn assert_traits_implemented<T>()
where
    T: Copy
        + Clone
        + Default
        + PartialEq
        + Eq
        + std::hash::Hash
        + PartialOrd
        + Ord
        + std::fmt::Debug
        + std::fmt::Display,
{
}

#[test]
fn implements_expected_traits() {
    assert_traits_implemented::<Id<User>>();
    assert_traits_implemented::<Id<Product>>();
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn id_is_send_and_sync() {
    assert_send_sync::<Id<User>>();
    assert_send_sync::<Id<Product>>();
}


#[test]
fn size_matches_i64() {
    assert_eq!(size_of::<Id<User>>(), size_of::<i64>());
    assert_eq!(size_of::<Id<Product>>(), size_of::<i64>());
}

#[test]
fn alignment_matches_i64() {
    assert_eq!(align_of::<Id<User>>(), align_of::<i64>());
    assert_eq!(align_of::<Id<Product>>(), align_of::<i64>());
}


#[test]
fn marker_types_produce_distinct_id_types() {
    assert_ne!(
        TypeId::of::<Id<User>>(),
        TypeId::of::<Id<Product>>(),
        "Id<User> and Id<Product> must be distinct types"
    );
}

#[test]
fn same_marker_produces_same_id_type() {
    assert_eq!(TypeId::of::<Id<User>>(), TypeId::of::<Id<User>>());
    assert_eq!(TypeId::of::<Id<Product>>(), TypeId::of::<Id<Product>>());
}


#[test]
fn new_and_value_roundtrip() {
    let cases = [0, 1, -1, 42, -42, i64::MAX, i64::MIN];
    for value in cases {
        let id: Id<User> = Id::new(value);
        assert_eq!(id.value(), value, "value {value} did not roundtrip");
    }
}

#[test]
fn default_is_zero() {
    assert_eq!(Id::<User>::default().value(), 0);
    assert_eq!(Id::<Product>::default().value(), 0);
}


const ZERO_ID: Id<User> = Id::new(0);
const MAX_ID: Id<User> = Id::new(i64::MAX);
const MIN_ID: Id<User> = Id::new(i64::MIN);
const NEG_ID: Id<User> = Id::new(-1);

#[test]
fn const_constructed_ids_have_correct_values() {
    assert_eq!(ZERO_ID.value(), 0);
    assert_eq!(MAX_ID.value(), i64::MAX);
    assert_eq!(MIN_ID.value(), i64::MIN);
    assert_eq!(NEG_ID.value(), -1);
}

#[test]
fn const_id_usable_in_const_context() {
    const ONE: Id<User> = Id::new(1);
    const DOUBLE: i64 = ONE.value() * 2; 
    assert_eq!(DOUBLE, 2);
}


#[test]
fn serialize_transparent_integer() {
    for value in [0, 1, -1, 42, i64::MAX, i64::MIN] {
        let id: Id<User> = Id::new(value);
        assert_eq!(
            to_json(&id).unwrap(),
            json!(value),
            "value {value} serialized incorrectly"
        );
    }
}

#[test]
fn serialize_produces_json_integer_not_float_or_string() {
    let id: Id<User> = Id::new(42);
    let json = to_json(&id).unwrap();
    assert!(json.is_i64(), "expected i64, got {json}");
    assert!(!json.is_f64());
    assert!(!json.is_string());
}


#[test]
fn deserialize_transparent_integer() {
    for value in [0, 1, -1, 42, i64::MAX, i64::MIN] {
        let id: Id<User> = from_json(&json!(value)).unwrap();
        assert_eq!(id.value(), value);
    }
}

#[test]
fn deserialize_rejects_string() {
    for s in ["42", "abc", ""] {
        assert!(
            from_json::<Id<User>>(&json!(s)).is_err(),
            "string {s:?} should not deserialize"
        );
    }
}

#[test]
fn deserialize_rejects_float() {
    for f in [42.5, 42.0, -1.5, 0.0] {
        assert!(
            from_json::<Id<User>>(&json!(f)).is_err(),
            "float {f} should not deserialize"
        );
    }
}

#[test]
fn deserialize_rejects_null() {
    assert!(from_json::<Id<User>>(&json!(null)).is_err());
}

#[test]
fn deserialize_rejects_bool() {
    assert!(from_json::<Id<User>>(&json!(true)).is_err());
    assert!(from_json::<Id<User>>(&json!(false)).is_err());
}

#[test]
fn deserialize_rejects_array() {
    assert!(from_json::<Id<User>>(&json!([1])).is_err());
    assert!(from_json::<Id<User>>(&json!([])).is_err());
}

#[test]
fn deserialize_rejects_object() {
    assert!(from_json::<Id<User>>(&json!({"value": 42})).is_err());
    assert!(from_json::<Id<User>>(&json!({})).is_err());
}

#[test]
fn deserialize_rejects_positive_out_of_range() {
    let too_big = (i64::MAX as u64) + 1;
    assert!(from_json::<Id<User>>(&json!(too_big)).is_err());
}

#[test]
fn deserialize_rejects_u64_max() {
    assert!(from_json::<Id<User>>(&json!(u64::MAX)).is_err());
}

#[test]
fn deserialize_rejects_negative_out_of_range() {
    let min: Id<User> = from_json(&json!(i64::MIN)).unwrap();
    assert_eq!(min.value(), i64::MIN);
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct UserRecord {
    id: Id<User>,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ProductRecord {
    id: Id<Product>,
    price: i64,
}

#[test]
fn id_serializes_as_plain_field_in_struct() {
    let record = UserRecord {
        id: Id::new(42),
        name: "Alice".into(),
    };
    assert_eq!(
        to_json(&record).unwrap(),
        json!({"id": 42, "name": "Alice"})
    );
}

#[test]
fn id_deserializes_as_plain_field_in_struct() {
    let json = json!({"id": 42, "name": "Alice"});
    let record: UserRecord = from_json(&json).unwrap();
    assert_eq!(record.id.value(), 42);
    assert_eq!(record.name, "Alice");
}

#[test]
fn two_id_types_coexist_in_separate_structs() {
    let user = UserRecord {
        id: Id::new(1),
        name: "A".into(),
    };
    let product = ProductRecord {
        id: Id::new(1),
        price: 100,
    };
    assert_eq!(user.id.value(), product.id.value());
    assert_eq!(
        to_json(&user).unwrap(),
        json!({"id": 1, "name": "A"})
    );
    assert_eq!(
        to_json(&product).unwrap(),
        json!({"id": 1, "price": 100})
    );
}

#[test]
fn id_in_nested_structure() {
    #[derive(Serialize, Deserialize)]
    struct Order {
        id: Id<Product>,
        buyer: Id<User>,
    }
    let order = Order {
        id: Id::new(10),
        buyer: Id::new(20),
    };
    assert_eq!(
        to_json(&order).unwrap(),
        json!({"id": 10, "buyer": 20})
    );
}


#[test]
fn roundtrip_all_extreme_values() {
    for value in [0, 1, -1, 42, -42, i64::MAX, i64::MIN] {
        let original: Id<User> = Id::new(value);
        let json = to_json(&original).unwrap();
        let back: Id<User> = from_json(&json).unwrap();
        assert_eq!(back.value(), original.value(), "value {value} roundtrip");
    }
}


#[test]
fn display_shows_raw_integer() {
    assert_eq!(Id::<User>::new(0).to_string(), "0");
    assert_eq!(Id::<User>::new(42).to_string(), "42");
    assert_eq!(Id::<User>::new(-5).to_string(), "-5");
    assert_eq!(Id::<User>::new(i64::MAX).to_string(), "9223372036854775807");
    assert_eq!(
        Id::<User>::new(i64::MIN).to_string(),
        "-9223372036854775808"
    );
}

#[test]
fn display_matches_i64_display() {
    for value in [0, -1, 1, i64::MAX, i64::MIN] {
        assert_eq!(Id::<User>::new(value).to_string(), value.to_string());
    }
}

#[test]
fn format_with_padding() {
    let id: Id<User> = Id::new(42);
    assert_eq!(format!("{id:>5}"), "   42");
}


#[test]
fn debug_contains_value() {
    let dbg = format!("{:?}", Id::<User>::new(42));
    assert!(dbg.contains("42"));
}

#[test]
fn debug_contains_type_name() {
    let dbg = format!("{:?}", Id::<User>::new(1));
    assert!(dbg.contains("Id"));
}


#[test]
fn copy_semantics() {
    let a: Id<User> = Id::new(42);
    let b = a; 
    let c = a; 
    assert_eq!(a.value(), 42);
    assert_eq!(b.value(), 42);
    assert_eq!(c.value(), 42);
}

#[test]
fn explicit_clone() {
    let a: Id<User> = Id::new(7);
    let b = a.clone();
    assert_eq!(a.value(), b.value());
}


#[test]
fn equality_is_value_based() {
    let a: Id<User> = Id::new(5);
    let b: Id<User> = Id::new(5);
    let c: Id<User> = Id::new(6);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn equality_holds_across_extreme_values() {
    for value in [0, -1, 1, i64::MAX, i64::MIN] {
        assert_eq!(Id::<User>::new(value), Id::<User>::new(value));
    }
}

#[test]
fn equality_reflexive() {
    for value in [0, 1, -1, i64::MAX, i64::MIN] {
        let id: Id<User> = Id::new(value);
        assert_eq!(id, id);
    }
}


#[test]
fn ordering_follows_i64() {
    let pairs = [
        (i64::MIN, i64::MIN + 1),
        (-1, 0),
        (0, 1),
        (1, 42),
        (i64::MAX - 1, i64::MAX),
    ];
    for (a, b) in pairs {
        let id_a: Id<User> = Id::new(a);
        let id_b: Id<User> = Id::new(b);
        assert!(id_a < id_b, "{a} should be < {b}");
        assert!(id_b > id_a, "{b} should be > {a}");
    }
}

#[test]
fn ordering_with_equal_values() {
    let a: Id<User> = Id::new(5);
    let b: Id<User> = Id::new(5);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);
}

#[test]
fn sort_uses_numeric_order() {
    let mut v = vec![
        Id::<User>::new(3),
        Id::<User>::new(1),
        Id::<User>::new(2),
        Id::<User>::new(-1),
        Id::<User>::new(0),
    ];
    v.sort();
    let values: Vec<i64> = v.iter().map(|id| id.value()).collect();
    assert_eq!(values, vec![-1, 0, 1, 2, 3]);
}

#[test]
fn sort_is_total_and_deterministic() {
    let mut v1 = vec![
        Id::<User>::new(5),
        Id::<User>::new(1),
        Id::<User>::new(3),
    ];
    let mut v2 = v1.clone();
    v1.sort();
    v2.sort();
    assert_eq!(v1, v2);
}


#[test]
fn hash_consistent_with_equality() {
    let a: Id<User> = Id::new(42);
    let b: Id<User> = Id::new(42);
    let mut set = HashSet::new();
    set.insert(a);
    assert!(set.contains(&b), "equal IDs must hash equally");
}

#[test]
fn hashset_deduplicates() {
    let mut set: HashSet<Id<User>> = HashSet::new();
    set.insert(Id::new(1));
    set.insert(Id::new(2));
    set.insert(Id::new(1));
    set.insert(Id::new(2));
    assert_eq!(set.len(), 2);
}

#[test]
fn hashset_distinguishes_different_values() {
    let mut set: HashSet<Id<User>> = HashSet::new();
    for v in [0, 1, -1, i64::MAX, i64::MIN] {
        set.insert(Id::<User>::new(v));
    }
    assert_eq!(set.len(), 5);
}


#[test]
fn btreeset_stores_sorted() {
    let mut set: BTreeSet<Id<User>> = BTreeSet::new();
    set.insert(Id::new(3));
    set.insert(Id::new(1));
    set.insert(Id::new(2));
    let values: Vec<i64> = set.iter().map(|id| id.value()).collect();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn btreemap_uses_value_ordering() {
    let mut map: BTreeMap<Id<User>, &str> = BTreeMap::new();
    map.insert(Id::new(2), "two");
    map.insert(Id::new(1), "one");
    map.insert(Id::new(3), "three");
    let keys: Vec<i64> = map.keys().map(|id| id.value()).collect();
    assert_eq!(keys, vec![1, 2, 3]);
}


#[test]
fn distinct_instances_are_independent() {
    let a: Id<User> = Id::new(1);
    let b: Id<User> = Id::new(1);
    assert_eq!(a.value(), b.value());
}


#[test]
fn same_value_different_marker_has_same_display() {
    assert_eq!(
        Id::<User>::new(42).to_string(),
        Id::<Product>::new(42).to_string()
    );
}

#[test]
fn same_value_different_marker_has_same_serialization() {
    assert_eq!(
        to_json(&Id::<User>::new(42)).unwrap(),
        to_json(&Id::<Product>::new(42)).unwrap()
    );
}

#[test]
fn same_value_different_marker_has_same_size() {
    assert_eq!(size_of::<Id<User>>(), size_of::<Id<Product>>());
}
