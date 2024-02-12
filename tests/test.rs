use addr_of_enum::{addr_of_enum, get_discriminant, AddrOfEnum};
use core::convert::Infallible;

#[derive(AddrOfEnum)]
#[repr(C)]
// Disturb rustfmt
#[derive(PartialEq, Eq)]
enum E<T> {
    E1(usize, u8, u16),
    E2 {
        item1: u32,
        item2: T,
    },
    #[allow(unused)]
    E3,
}
#[test]
fn test() {
    let e1: E<u8> = E::E1(1, 2, 3);
    let e2 = E::E2 {
        item1: 1,
        item2: 2u8,
    };
    let e1a: E<Infallible> = E::E1(1, 2, 3);
    match &e1 {
        E::E1(item1, item2, item3) => {
            assert_eq!(item1 as *const usize, addr_of_enum!(&e1, E1, 0));
            assert_eq!(item2 as *const u8, addr_of_enum!(&e1, E1, 1));
            assert_eq!(item3 as *const u16, addr_of_enum!(&e1, E1, 2));
        }
        _ => panic!(),
    }
    match &e2 {
        E::E2 { item1, item2 } => {
            assert_eq!(item1 as *const u32, addr_of_enum!(&e2, E2, item1));
            assert_eq!(item2 as *const u8, addr_of_enum!(&e2, E2, item2));
        }
        _ => panic!(),
    }
    // uninhabited
    match &e1a {
        E::E1(item1, item2, item3) => {
            assert_eq!(item1 as *const usize, addr_of_enum!(&e1a, E1, 0));
            assert_eq!(item2 as *const u8, addr_of_enum!(&e1a, E1, 1));
            assert_eq!(item3 as *const u16, addr_of_enum!(&e1a, E1, 2));
        }
        _ => panic!(),
    }
    assert_eq!(core::mem::discriminant(&e1), get_discriminant!(E<u8>, E1));
    assert_eq!(core::mem::discriminant(&e2), get_discriminant!(E<u8>, E2));
}
