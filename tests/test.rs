use enum_offset::{enum_offset, EnumOffset};

#[derive(EnumOffset)]
enum E<T> {
    E1(usize, u8, u16),
    E2 { item1: u32, item2: T },
}
#[test]
fn test() {
    let e1: E<u8> = E::E1(1, 2, 3);
    let e2 = E::E2 {
        item1: 1,
        item2: 2u8,
    };
    match &e1 {
        E::E1(item1, item2, item3) => {
            assert_eq!(item1 as *const usize, enum_offset!(&e1, E1, 0));
            assert_eq!(item2 as *const u8, enum_offset!(&e1, E1, 1));
            assert_eq!(item3 as *const u16, enum_offset!(&e1, E1, 2));
        }
        _ => panic!(),
    }
    match &e2 {
        E::E2 { item1, item2 } => {
            assert_eq!(item1 as *const u32, enum_offset!(&e2, E2, item1));
            assert_eq!(item2 as *const u8, enum_offset!(&e2, E2, item2));
        }
        _ => panic!(),
    }
}
