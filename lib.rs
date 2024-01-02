#![doc = include_str!("README.md")]

pub use addr_of_enum_macro::AddrOfEnum;

#[doc(hidden)]
pub mod macro_def {
    pub use addr_of_enum_macro::get_tstr;

    /// See crate level documentation
    #[macro_export]
    macro_rules! addr_of_enum {
        ($e:expr, $tag:ident, $field:tt) => {
            <_ as $crate::EnumHasTagAndField<
                $crate::macro_def::get_tstr!($crate, $tag),
                $crate::macro_def::get_tstr!($crate, $field),
            >>::addr_of($e as *const _)
        };
    }

    #[macro_export]
    macro_rules! get_discriminant {
        ($enum_ty:ty, $tag:ident) => {
            <$enum_ty as $crate::EnumHasTag<$crate::macro_def::get_tstr!($crate, $tag)>>::discriminant()
        };
    }
}

/// This trait is implemented with `#[derive(AddrOfEnum)]`
pub unsafe trait AddrOfEnum: Sized {}

#[doc(hidden)]
pub unsafe trait EnumHasTag<TSTag>: AddrOfEnum {
    fn discriminant() -> core::mem::Discriminant<Self>;
}

#[doc(hidden)]
pub unsafe trait EnumHasTagAndField<TSTag, TSField>: EnumHasTag<TSTag> {
    type Ty: Sized;
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
