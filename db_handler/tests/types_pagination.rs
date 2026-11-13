#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::constants::DEFAULT_PAGE_SIZE;
use db_handler::json::{from_json, to_json, Json};
use db_handler::types::core::{PageMeta, PageRequest, Paginated};
use serde_json::json;

#[test]
fn page_request_default_page_is_one() {
    let req = PageRequest::default();
    assert_eq!(req.page, 1);
}

#[test]
fn page_request_default_per_page_is_default_constant() {
    let req = PageRequest::default();
    assert_eq!(req.per_page, DEFAULT_PAGE_SIZE);
}

#[test]
fn page_request_deserialize_empty_object_uses_defaults() {
    let req: PageRequest = from_json(&json!({})).unwrap();
    assert_eq!(req.page, 1);
    assert_eq!(req.per_page, DEFAULT_PAGE_SIZE);
}

#[test]
fn page_request_deserialize_only_page() {
    let req: PageRequest = from_json(&json!({"page": 5})).unwrap();
    assert_eq!(req.page, 5);
    assert_eq!(req.per_page, DEFAULT_PAGE_SIZE);
}

#[test]
fn page_request_deserialize_only_per_page() {
    let req: PageRequest = from_json(&json!({"per_page": 50})).unwrap();
    assert_eq!(req.page, 1);
    assert_eq!(req.per_page, 50);
}

#[test]
fn page_request_deserialize_both_fields() {
    let req: PageRequest = from_json(&json!({"page": 3, "per_page": 25})).unwrap();
    assert_eq!(req.page, 3);
    assert_eq!(req.per_page, 25);
}

#[test]
fn page_request_deserialize_zero_page_allowed() {
    let req: PageRequest = from_json(&json!({"page": 0})).unwrap();
    assert_eq!(req.page, 0);
}

#[test]
fn page_request_deserialize_zero_per_page_allowed() {
    let req: PageRequest = from_json(&json!({"per_page": 0})).unwrap();
    assert_eq!(req.per_page, 0);
}

#[test]
fn page_request_deserialize_u64_max_allowed() {
    let req: PageRequest = from_json(&json!({
        "page": u64::MAX,
        "per_page": u64::MAX
    }))
    .unwrap();
    assert_eq!(req.page, u64::MAX);
    assert_eq!(req.per_page, u64::MAX);
}

#[test]
fn page_request_rejects_negative_page() {
    assert!(from_json::<PageRequest>(&json!({"page": -1})).is_err());
}

#[test]
fn page_request_rejects_negative_per_page() {
    assert!(from_json::<PageRequest>(&json!({"per_page": -1})).is_err());
}

#[test]
fn page_request_rejects_string_values() {
    assert!(from_json::<PageRequest>(&json!({"page": "5"})).is_err());
    assert!(from_json::<PageRequest>(&json!({"per_page": "50"})).is_err());
}

#[test]
fn page_request_rejects_float_values() {
    assert!(from_json::<PageRequest>(&json!({"page": 1.5})).is_err());
    assert!(from_json::<PageRequest>(&json!({"per_page": 1.5})).is_err());
}

#[test]
fn page_request_rejects_null_page() {
    assert!(from_json::<PageRequest>(&json!({"page": null})).is_err());
}

#[test]
fn page_request_accepts_extra_fields() {
    let req: PageRequest = from_json(&json!({"page": 2, "extra": "x"})).unwrap();
    assert_eq!(req.page, 2);
}

#[test]
fn page_request_rejects_null_root() {
    assert!(from_json::<PageRequest>(&json!(null)).is_err());
}

#[test]
fn page_request_serialize_exact_shape() {
    let req = PageRequest {
        page: 2,
        per_page: 30,
    };
    let j = to_json(&req).unwrap();
    let obj = j.as_object().unwrap();
    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("page").and_then(Json::as_u64), Some(2));
    assert_eq!(obj.get("per_page").and_then(Json::as_u64), Some(30));
}

#[test]
fn page_request_roundtrip() {
    let original = PageRequest {
        page: 7,
        per_page: 42,
    };
    let back: PageRequest = from_json(&to_json(&original).unwrap()).unwrap();
    assert_eq!(back.page, original.page);
    assert_eq!(back.per_page, original.per_page);
}

#[test]
fn page_request_offset_first_page_is_zero() {
    let req = PageRequest {
        page: 1,
        per_page: 20,
    };
    assert_eq!(req.offset(), 0);
}

#[test]
fn page_request_offset_second_page() {
    let req = PageRequest {
        page: 2,
        per_page: 20,
    };
    assert_eq!(req.offset(), 20);
}

#[test]
fn page_request_offset_third_page_custom() {
    let req = PageRequest {
        page: 3,
        per_page: 5,
    };
    assert_eq!(req.offset(), 10);
}

#[test]
fn page_request_offset_zero_page_saturates() {
    let req = PageRequest {
        page: 0,
        per_page: 20,
    };
    assert_eq!(req.offset(), 0);
}

#[test]
fn page_request_offset_huge_values_saturate() {
    let req = PageRequest {
        page: u64::MAX,
        per_page: u64::MAX,
    };
    assert_eq!(req.offset(), u64::MAX);
}

#[test]
fn page_request_offset_zero_per_page() {
    let req = PageRequest {
        page: 5,
        per_page: 0,
    };
    assert_eq!(req.offset(), 0);
}

#[test]
fn page_request_copy() {
    let a = PageRequest {
        page: 1,
        per_page: 10,
    };
    let b = a;
    assert_eq!(a.page, b.page);
    assert_eq!(a.per_page, b.per_page);
}

#[test]
fn page_request_debug_contains_type_name() {
    let req = PageRequest {
        page: 1,
        per_page: 20,
    };
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("PageRequest"));
    assert!(dbg.contains("page"));
}

#[test]
fn page_meta_zero_total() {
    let m = PageMeta::new(1, 20, 0);
    assert_eq!(m.total_pages, 0);
}

#[test]
fn page_meta_exact_multiple() {
    let m1 = PageMeta::new(1, 20, 20);
    assert_eq!(m1.total_pages, 1);
    let m2 = PageMeta::new(1, 20, 40);
    assert_eq!(m2.total_pages, 2);
    let m3 = PageMeta::new(1, 20, 100);
    assert_eq!(m3.total_pages, 5);
}

#[test]
fn page_meta_remainder_rounds_up() {
    let m1 = PageMeta::new(1, 20, 1);
    assert_eq!(m1.total_pages, 1);
    let m2 = PageMeta::new(1, 20, 21);
    assert_eq!(m2.total_pages, 2);
    let m3 = PageMeta::new(1, 20, 41);
    assert_eq!(m3.total_pages, 3);
}

#[test]
fn page_meta_zero_per_page() {
    let m1 = PageMeta::new(1, 0, 100);
    assert_eq!(m1.total_pages, 0);
    let m2 = PageMeta::new(1, 0, 0);
    assert_eq!(m2.total_pages, 0);
}

#[test]
fn page_meta_copies_input_fields() {
    let m = PageMeta::new(3, 20, 100);
    assert_eq!(m.page, 3);
    assert_eq!(m.per_page, 20);
    assert_eq!(m.total, 100);
    assert_eq!(m.total_pages, 5);
}

#[test]
fn page_meta_u64_max_values() {
    let m = PageMeta::new(u64::MAX, u64::MAX, u64::MAX);
    assert_eq!(m.total_pages, 1);
}

#[test]
fn page_meta_serialize_exact_shape() {
    let m = PageMeta::new(2, 10, 25);
    let j = to_json(&m).unwrap();
    let obj = j.as_object().unwrap();
    assert_eq!(obj.len(), 4);
    assert_eq!(obj.get("page").and_then(Json::as_u64), Some(2));
    assert_eq!(obj.get("per_page").and_then(Json::as_u64), Some(10));
    assert_eq!(obj.get("total").and_then(Json::as_u64), Some(25));
    assert_eq!(obj.get("total_pages").and_then(Json::as_u64), Some(3));
}

#[test]
fn page_meta_roundtrip() {
    let original = PageMeta::new(2, 10, 25);
    let back: PageMeta = from_json(&to_json(&original).unwrap()).unwrap();
    assert_eq!(back.page, original.page);
    assert_eq!(back.per_page, original.per_page);
    assert_eq!(back.total, original.total);
    assert_eq!(back.total_pages, original.total_pages);
}

#[test]
fn page_meta_deserialize_ignores_computed_field() {
    let m: PageMeta = from_json(&json!({
        "page": 1,
        "per_page": 20,
        "total": 100,
        "total_pages": 999
    }))
    .unwrap();
    assert_eq!(m.total_pages, 999);
}

#[test]
fn page_meta_copy() {
    let a = PageMeta::new(1, 20, 100);
    let b = a;
    assert_eq!(a.page, b.page);
    assert_eq!(a.total_pages, b.total_pages);
}

#[test]
fn page_meta_debug_contains_type_name() {
    assert!(format!("{:?}", PageMeta::new(1, 20, 5)).contains("PageMeta"));
}

#[test]
fn paginated_empty_items_serialize() {
    let p: Paginated<i32> = Paginated {
        items: vec![],
        meta: PageMeta::new(1, 20, 0),
    };
    let j = to_json(&p).unwrap();
    let obj = j.as_object().unwrap();
    assert_eq!(obj.len(), 2);

    let items = obj.get("items").and_then(|v| v.as_array()).unwrap();
    assert!(items.is_empty());

    let meta = obj.get("meta").and_then(|v| v.as_object()).unwrap();
    assert_eq!(meta.get("total").and_then(Json::as_u64), Some(0));
}

#[test]
fn paginated_with_primitives_serialize() {
    let p: Paginated<i32> = Paginated {
        items: vec![1, 2, 3],
        meta: PageMeta::new(1, 20, 3),
    };
    let j = to_json(&p).unwrap();
    let items = j.get("items").and_then(|v| v.as_array()).unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items.first().and_then(Json::as_i64), Some(1));
    assert_eq!(items.get(1).and_then(Json::as_i64), Some(2));
    assert_eq!(items.get(2).and_then(Json::as_i64), Some(3));
}

#[test]
fn paginated_empty_roundtrip() {
    let original: Paginated<i32> = Paginated {
        items: vec![],
        meta: PageMeta::new(1, 20, 0),
    };
    let back: Paginated<i32> = from_json(&to_json(&original).unwrap()).unwrap();
    assert!(back.items.is_empty());
    assert_eq!(back.meta.total, 0);
}

#[test]
fn paginated_with_primitives_roundtrip() {
    let original: Paginated<i32> = Paginated {
        items: vec![1, 2, 3],
        meta: PageMeta::new(1, 20, 3),
    };
    let back: Paginated<i32> = from_json(&to_json(&original).unwrap()).unwrap();
    assert_eq!(back.items, vec![1, 2, 3]);
    assert_eq!(back.meta.total, 3);
}

#[test]
fn paginated_with_structs_roundtrip() {
    let original: Paginated<PageMeta> = Paginated {
        items: vec![PageMeta::new(1, 10, 5), PageMeta::new(2, 10, 5)],
        meta: PageMeta::new(1, 10, 2),
    };
    let back: Paginated<PageMeta> = from_json(&to_json(&original).unwrap()).unwrap();
    assert_eq!(back.items.len(), 2);
    assert_eq!(back.items.first().map(|x| x.page), Some(1));
    assert_eq!(back.items.get(1).map(|x| x.page), Some(2));
}

#[test]
fn paginated_with_strings_roundtrip() {
    let original: Paginated<String> = Paginated {
        items: vec!["a".into(), "b".into()],
        meta: PageMeta::new(1, 20, 2),
    };
    let back: Paginated<String> = from_json(&to_json(&original).unwrap()).unwrap();
    assert_eq!(back.items, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn paginated_rejects_missing_items() {
    let r = from_json::<Paginated<i32>>(&json!({
        "meta": {"page": 1, "per_page": 20, "total": 0, "total_pages": 0}
    }));
    assert!(r.is_err());
}

#[test]
fn paginated_rejects_missing_meta() {
    let r = from_json::<Paginated<i32>>(&json!({"items": [1, 2, 3]}));
    assert!(r.is_err());
}

#[test]
fn paginated_rejects_null_root() {
    let r = from_json::<Paginated<i32>>(&json!(null));
    assert!(r.is_err());
}

#[test]
fn paginated_clone_preserves_all() {
    let original: Paginated<i32> = Paginated {
        items: vec![1, 2, 3],
        meta: PageMeta::new(2, 15, 33),
    };
    let cloned = original.clone();
    assert_eq!(cloned.items, original.items);
    assert_eq!(cloned.meta.page, original.meta.page);
    assert_eq!(cloned.meta.total, original.meta.total);
}
