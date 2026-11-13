#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::constants::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use db_handler::json::{from_json, to_json};
use db_handler::types::core::{PageMeta, PageRequest, Paginated};
use serde_json::json;


#[test]
fn page_request_default_page() {
    assert_eq!(PageRequest::default().page, 1);
}

#[test]
fn page_request_default_per_page() {
    assert_eq!(PageRequest::default().per_page, DEFAULT_PAGE_SIZE);
}

#[test]
fn page_request_deserialize_empty_object() {
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
fn page_request_deserialize_both() {
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
}

#[test]
fn page_request_serialize_both_fields() {
    let req = PageRequest { page: 2, per_page: 30 };
    assert_eq!(to_json(&req).unwrap(), json!({"page": 2, "per_page": 30}));
}

#[test]
fn page_request_roundtrip() {
    let original = PageRequest { page: 7, per_page: 42 };
    let back: PageRequest = from_json(&to_json(&original).unwrap()).unwrap();
    assert_eq!(back.page, original.page);
    assert_eq!(back.per_page, original.per_page);
}


#[test]
fn page_request_offset_first_page_is_zero() {
    assert_eq!(PageRequest { page: 1, per_page: 20 }.offset(), 0);
}

#[test]
fn page_request_offset_second_page() {
    assert_eq!(PageRequest { page: 2, per_page: 20 }.offset(), 20);
}

#[test]
fn page_request_offset_third_page_custom_size() {
    assert_eq!(PageRequest { page: 3, per_page: 5 }.offset(), 10);
}

#[test]
fn page_request_offset_zero_page_saturates() {
    assert_eq!(PageRequest { page: 0, per_page: 20 }.offset(), 0);
}

#[test]
fn page_request_offset_huge_values_saturate() {
    assert_eq!(
        PageRequest { page: u64::MAX, per_page: u64::MAX }.offset(),
        u64::MAX
    );
}

#[test]
fn page_request_is_copy() {
    let a = PageRequest { page: 1, per_page: 10 };
    let b = a;
    assert_eq!(a.page, b.page);
}

#[test]
fn page_request_debug() {
    let dbg = format!("{:?}", PageRequest { page: 1, per_page: 20 });
    assert!(dbg.contains("PageRequest"));
    assert!(dbg.contains("page"));
}


#[test]
fn page_meta_zero_total() {
    assert_eq!(PageMeta::new(1, 20, 0).total_pages, 0);
}

#[test]
fn page_meta_exact_multiple() {
    assert_eq!(PageMeta::new(1, 20, 20).total_pages, 1);
    assert_eq!(PageMeta::new(1, 20, 40).total_pages, 2);
    assert_eq!(PageMeta::new(1, 20, 100).total_pages, 5);
}

#[test]
fn page_meta_remainder_rounds_up() {
    assert_eq!(PageMeta::new(1, 20, 1).total_pages, 1);
    assert_eq!(PageMeta::new(1, 20, 21).total_pages, 2);
    assert_eq!(PageMeta::new(1, 20, 41).total_pages, 3);
}

#[test]
fn page_meta_zero_per_page() {
    assert_eq!(PageMeta::new(1, 0, 100).total_pages, 0);
    assert_eq!(PageMeta::new(1, 0, 0).total_pages, 0);
}

#[test]
fn page_meta_copies_input() {
    let m = PageMeta::new(3, 20, 100);
    assert_eq!(m.page, 3);
    assert_eq!(m.per_page, 20);
    assert_eq!(m.total, 100);
    assert_eq!(m.total_pages, 5);
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
fn page_meta_is_copy() {
    let a = PageMeta::new(1, 20, 100);
    let b = a;
    assert_eq!(a.page, b.page);
}


#[test]
fn paginated_empty_items() {
    let p = Paginated::<i32> { items: vec![], meta: PageMeta::new(1, 20, 0) };
    let json = to_json(&p).unwrap();
    assert_eq!(json["items"].as_array().unwrap().len(), 0);
    assert_eq!(json["meta"]["total"].as_u64(), Some(0));
}

#[test]
fn paginated_with_primitive_items_roundtrip() {
    let original = Paginated {
        items: vec![1i32, 2, 3],
        meta: PageMeta::new(1, 20, 3),
    };
    let back: Paginated<i32> = from_json(&to_json(&original).unwrap()).unwrap();
    assert_eq!(back.items, vec![1, 2, 3]);
    assert_eq!(back.meta.total, 3);
}

#[test]
fn paginated_with_struct_items_roundtrip() {
    let original = Paginated {
        items: vec![PageMeta::new(1, 10, 5), PageMeta::new(2, 10, 5)],
        meta: PageMeta::new(1, 10, 2),
    };
    let back: Paginated<PageMeta> = from_json(&to_json(&original).unwrap()).unwrap();
    assert_eq!(back.items.len(), 2);
    assert_eq!(back.items[0].page, 1);
    assert_eq!(back.items[1].page, 2);
}
