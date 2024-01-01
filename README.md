# addr_of_enum

This crate provides `#[derive(AddrOfEnum)]` and `addr_of_enum!` macro to get a field pointer of specified variant without creating an intermediated reference. It works like `std::ptr::addr_of!`, but works only on enums.

This macro is zero-cost, which means that it calculates without minimum cost in release mode.

It also works on variables which has uninhabited types.

## Example

```rust
# use addr_of_enum::{addr_of_enum, AddrOfEnum};

#[derive(AddrOfEnum)]
enum MyEnum {
    E1(usize, u8),
    E2 {
        item1: u32,
        item2: u8,
    }
}

let e = MyEnum::E1(1, 2);
let _: *const usize = addr_of_enum!(&e, E1, 0);
let _: *const u32 = addr_of_enum!(&e, E2, item1);
```

## Limitations

For now, the macros cannot be used in const context.
