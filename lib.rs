#![doc = include_str!("README.md")]

pub use addr_of_enum_macro::AddrOfEnum;

#[doc(hidden)]
pub mod macro_def {
    pub use addr_of_enum_macro::addr_of_enum as macro_addr_of_enum;

    #[macro_export]
    macro_rules! addr_of_enum {
        ($e:expr, $tag:ident, $t:tt) => {
            $crate::macro_def::macro_addr_of_enum!($crate, $e, $tag, $t)
        };
    }
}

/// This trait is implemented with `#[derive(AddrOfEnum)]`
pub unsafe trait AddrOfEnum {}

#[doc(hidden)]
pub unsafe trait EnumHasTagAndField<TSTag, TSField>: AddrOfEnum {
    type Ty;
    fn addr_of(ptr: *const Self) -> *const Self::Ty;
}

#[doc(hidden)]
pub mod _tstr {
    macro_rules! chars {
        () => {};
        ($id:ident $($rem:tt)*) => {
            #[allow(non_camel_case_types)]
            pub struct $id(::core::convert::Infallible);
            chars!($($rem)*);
        };
    }
    chars! {
        _A _B _C _D _E _F _G _H _I _J _K _L _M _N _O _P _Q _R _S _T _U
        _V _W _X _Y _Z
        _a _b _c _d _e _f _g _h _i _j _k _l _m _n _o _p _q _r _s _t _u
        _v _w _x _y _z
        _0 _1 _2 _3 _4 _5 _6 _7 _8 _9
        __
    }
}
