use core::convert::TryInto;

use super::*;

#[test]
fn from_vec() {
    assert!(BoundedVec::<u8, 2, 8>::from_vec(vec![1, 2]).is_ok());
    assert!(BoundedVec::<u8, 2, 8>::from_vec(vec![]).is_err());
    assert!(BoundedVec::<u8, 3, 8>::from_vec(vec![1, 2]).is_err());
    assert!(BoundedVec::<u8, 1, 2>::from_vec(vec![1, 2, 3]).is_err());
}

#[test]
fn is_empty() {
    let data: EmptyBoundedVec<_, 8> = vec![1u8, 2].try_into().unwrap();
    assert!(!data.is_empty());
}

#[test]
fn as_vec() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.as_vec(), &vec![1u8, 2]);
}

#[test]
fn as_slice() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.as_slice(), &[1u8, 2]);
}

#[test]
fn len() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.len(), 2);
}

#[test]
fn first() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.first(), &1u8);
}

#[test]
fn last() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.last(), &2u8);
}

#[test]
fn mapped() {
    let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    let data = data.mapped(|x| x * 2);
    assert_eq!(data, [2u8, 4].into());
}

#[test]
fn mapped_ref() {
    let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    let data = data.mapped_ref(|x| x * 2);
    assert_eq!(data, [2u8, 4].into());
}

#[test]
fn get() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.get(1).unwrap(), &2u8);
    assert!(data.get(3).is_none());
}

#[test]
fn try_mapped() {
    let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    let data = data.try_mapped(|x| 100u8.checked_div(x).ok_or("error"));
    assert_eq!(data, Ok([100u8, 50].into()));
}

#[test]
fn try_mapped_error() {
    let data: BoundedVec<u8, 2, 8> = [0u8, 2].into();
    let data = data.try_mapped(|x| 100u8.checked_div(x).ok_or("error"));
    assert_eq!(data, Err("error"));
}

#[test]
fn try_mapped_ref() {
    let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    let data = data.try_mapped_ref(|x| 100u8.checked_div(*x).ok_or("error"));
    assert_eq!(data, Ok([100u8, 50].into()));
}

#[test]
fn try_mapped_ref_error() {
    let data: BoundedVec<u8, 2, 8> = [0u8, 2].into();
    let data = data.try_mapped_ref(|x| 100u8.checked_div(*x).ok_or("error"));
    assert_eq!(data, Err("error"));
}

#[test]
fn split_last() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.split_last(), (&2u8, [1u8].as_ref()));
    let data1: BoundedVec<_, 1, 8> = vec![1u8].try_into().unwrap();
    assert_eq!(data1.split_last(), (&1u8, Vec::new().as_ref()));
}

#[test]
fn enumerated() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    let expected: BoundedVec<_, 2, 8> = vec![(0, 1u8), (1, 2)].try_into().unwrap();
    assert_eq!(data.enumerated(), expected);
}

#[test]
fn into_iter() {
    let mut vec = vec![1u8, 2];
    let mut data: BoundedVec<_, 2, 8> = vec.clone().try_into().unwrap();
    assert_eq!(data.clone().into_iter().collect::<Vec<u8>>(), vec);
    assert_eq!(
        data.iter().collect::<Vec<&u8>>(),
        vec.iter().collect::<Vec<&u8>>()
    );
    assert_eq!(
        data.iter_mut().collect::<Vec<&mut u8>>(),
        vec.iter_mut().collect::<Vec<&mut u8>>()
    );
}

#[cfg(feature = "borsh")]
mod borsh_tests {
    use borsh::schema::BorshSchemaContainer;
    use super::*;
    
    #[test]
    #[allow(clippy::expect_used)]
    fn borsh_encdec() {
        let data: BoundedVec<u8, 2, 8> = vec![1u8, 2].try_into().expect("borsh works");
        let buf = &mut Vec::new();
        data.serialize(buf).expect("borsh works");
        let decoded =
            BoundedVec::<u8, 2, 8>::deserialize(&mut buf.as_slice()).expect("borsh works");
        let compatible_decoded =
            BoundedVec::<u8, 1, 255>::deserialize(&mut buf.as_slice()).expect("borsh works");
        assert_eq!(data.get(0), decoded.get(0));
        assert_eq!(data.get(1), decoded.get(1));
        assert_eq!(data.get(0), compatible_decoded.get(0));
        assert_eq!(data.get(1), compatible_decoded.get(1));
        assert!(BoundedVec::<u8, 1, 257>::deserialize(&mut buf.as_slice()).is_err());

        let schema = BorshSchemaContainer::for_type::<BoundedVec<u8, 2, 8>>();
        let schema = schema
            .get_definition("BoundedVec<u8, 2, 8>")
            .expect("borsh works");
        assert!(matches!(
            schema,
            borsh::schema::Definition::Sequence {
                length_width: 1,
                ..
            }
        ));
    }
}

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;
    use alloc::vec;
    
    #[test]
    fn deserialize_nonempty() {
        assert_eq!(
            serde_json::from_str::<BoundedVec::<u8, 2, 3>>("[2, 3]")
                .unwrap()
                .as_vec(),
            &vec![2, 3]
        );
    }

    #[test]
    fn deserialize_empty() {
        assert!(serde_json::from_str::<EmptyBoundedVec::<u8, 3>>("[]").is_ok());
        assert!(serde_json::from_str::<BoundedVec::<u8, 2, 3>>("[]").is_err());
    }
}

#[cfg(feature = "schemars")]
mod schema_tests {
    use super::*;
    use schemars::schema_for;
    
    #[test]
    fn json_schema() {
        let root_schema = schema_for!(BoundedVec<u8, 2, 8>);
        let schema_value = serde_json::to_value(&root_schema).unwrap();
        let min_items = schema_value["minItems"].as_u64().unwrap() as u32;
        let max_items = schema_value["maxItems"].as_u64().unwrap() as u32;
        assert_eq!(min_items, 2);
        assert_eq!(max_items, 8);
    }
}

#[cfg(feature = "arbitrary")]
mod arb_tests {
    use super::*;
    use alloc::format;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn const_bounded_collections_length_bounded(v: BoundedVec<u8, 1, 2>) {
            prop_assert!(1 <= v.len() && v.len() <= 2);
        }
    }
}