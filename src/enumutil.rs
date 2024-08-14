use std::fmt::{Display, Formatter, Result as FmtResult};

pub trait RawEnum<I>: Sized {
    fn from_raw(raw: I) -> Option<Self>;
    fn to_raw(self) -> I;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enum<E, I> {
    Known(E),
    Unknown(I),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownEnumError<I> {
    value: I,
}

impl<
        E: std::fmt::Debug + std::cmp::PartialEq + std::marker::Copy + RawEnum<I>,
        I: std::fmt::Display + std::marker::Copy,
    > Enum<E, I>
{
    pub fn unwrap(self) -> E {
        match self {
            Enum::<E, I>::Known(k) => k,
            Enum::<E, I>::Unknown(u) => {
                panic!("unknown value {} in {}", u, std::any::type_name::<E>())
            }
        }
    }

    pub fn unwrap_or(self, def: E) -> E {
        match self {
            Enum::<E, I>::Known(k) => k,
            Enum::<E, I>::Unknown(_) => def,
        }
    }

    pub fn is_known(&self) -> bool {
        match self {
            Enum::<E, I>::Known(_) => true,
            Enum::<E, I>::Unknown(_) => false,
        }
    }

    pub fn is_known_and(self, f: impl FnOnce(E) -> bool) -> bool {
        match self {
            Enum::<E, I>::Known(k) => f(k),
            Enum::<E, I>::Unknown(_) => false,
        }
    }

    pub fn is_unknown(&self) -> bool {
        match self {
            Enum::<E, I>::Known(_) => false,
            Enum::<E, I>::Unknown(_) => true,
        }
    }

    pub fn is_unknown_and(self, f: impl FnOnce(I) -> bool) -> bool {
        match self {
            Enum::<E, I>::Known(_) => false,
            Enum::<E, I>::Unknown(u) => f(u),
        }
    }

    pub fn is(&self, want: E) -> bool {
        match self {
            Enum::<E, I>::Known(k) => want == *k,
            Enum::<E, I>::Unknown(_) => false,
        }
    }

    pub fn is_any(&self, want: Vec<E>) -> bool {
        match self {
            Enum::<E, I>::Known(k) => want.contains(k),
            Enum::<E, I>::Unknown(_) => false,
        }
    }

    pub fn known(self) -> Option<E> {
        match self {
            Enum::<E, I>::Known(k) => Some(k),
            Enum::<E, I>::Unknown(_) => None,
        }
    }

    pub fn unknown(self) -> Option<I> {
        match self {
            Enum::<E, I>::Known(_) => None,
            Enum::<E, I>::Unknown(u) => Some(u),
        }
    }

    /// Converts libvirt C enum constant to Rust enum.
    pub fn from_raw(raw: I) -> Enum<E, I> {
        match E::from_raw(raw) {
            Some(e) => Enum::<E, I>::Known(e),
            None => Enum::<E, I>::Unknown(raw),
        }
    }

    /// Converts Rust enum to libvirt C enum constant.
    pub fn to_raw(self) -> I {
        match self {
            Enum::Known(e) => e.to_raw(),
            Enum::Unknown(v) => v,
        }
    }
}

impl<I> UnknownEnumError<I> {
    pub fn value(self) -> I {
        self.value
    }
}

impl<E, I> From<E> for Enum<E, I> {
    fn from(v: E) -> Enum<E, I> {
        Enum::<E, I>::Known(v)
    }
}

impl<E, I> From<Enum<E, I>> for Result<E, UnknownEnumError<I>> {
    fn from(e: Enum<E, I>) -> Self {
        match e {
            Enum::<E, I>::Known(e) => Ok(e),
            Enum::<E, I>::Unknown(v) => Err(UnknownEnumError::<I> { value: v }),
        }
    }
}

impl<E: std::fmt::Display, I: std::fmt::Debug + std::fmt::Display> Display for Enum<E, I> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Enum::<E, I>::Known(e) => write!(f, "{}", e),
            Enum::<E, I>::Unknown(v) => write!(f, "{}({})", std::any::type_name::<E>(), v,),
        }
    }
}

macro_rules! impl_enum {
    (enum: $type:ty, raw: $raw:ty, match: { $($match_arms:tt)* }) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                $crate::enumutil::impl_enum_display!(self, f, $($match_arms)*)
            }
        }

        impl $crate::enumutil::RawEnum<$raw> for $type {
            /// Converts libvirt C enum constant to Rust enum.
            fn from_raw(raw: $raw) -> Option<Self> {
                $crate::enumutil::impl_enum_from!(raw, $($match_arms)*)
            }

            /// Converts Rust enum to libvirt C enum constant.
            fn to_raw(self) -> $raw {
                $crate::enumutil::impl_enum_to!(self, $($match_arms)*)
            }
        }
    }
}

macro_rules! impl_enum_display {
    (@acc ($e:expr, $f:expr, $(#[$attr:meta])* $raw:path => $type:ident,) -> ($($body:tt)*)) => {
        $crate::enumutil::impl_enum_display!(@final ($e) -> ($($body)* $(#[$attr])* Self::$type => write!($f, "{}", stringify!($type).to_lowercase())))
    };
    (@acc ($e:expr, $f:expr, $(#[$attr:meta])* $raw:path => $type:ident, $($match_arms:tt)*) -> ($($body:tt)*)) => {
        $crate::enumutil::impl_enum_display!(@acc ($e, $f, $($match_arms)*) -> ($($body)* $(#[$attr])* Self::$type => write!($f, "{}", stringify!($type).to_lowercase()),))
    };
    (@final ($e:expr) -> ($($body:tt)*)) => {
        match $e { $($body)* }
    };
    ($e:expr, $f:expr, $($match_arms:tt)*) => {
        $crate::enumutil::impl_enum_display!(@acc ($e, $f, $($match_arms)*) -> ())
    };
}

macro_rules! impl_enum_from {
    (@acc ($e:expr, $(#[$attr:meta])* $raw:path => $type:ident,) -> ($($body:tt)*)) => {
        $crate::enumutil::impl_enum_from!(@final ($e) -> ($($body)* $(#[$attr])* $raw => Some(Self::$type)))
    };
    (@acc ($e:expr, $(#[$attr:meta])* $raw:path => $type:ident, $($match_arms:tt)*) -> ($($body:tt)*)) => {
        $crate::enumutil::impl_enum_from!(@acc ($e, $($match_arms)*) -> ($($body)* $(#[$attr])* $raw => Some(Self::$type),))
    };
    (@final ($e:expr) -> ($($body:tt)*)) => {
        match $e { $($body)*, _ => None }
    };
    ($e:expr, $($match_arms:tt)*) => {
        $crate::enumutil::impl_enum_from!(@acc ($e, $($match_arms)*) -> ())
    };
}

macro_rules! impl_enum_to {
    (@acc ($e:expr, $(#[$attr:meta])* $raw:path => $type:ident,) -> ($($body:tt)*)) => {
        $crate::enumutil::impl_enum_to!(@final ($e) -> ($($body)* $(#[$attr])* Self::$type => $raw,))
    };
    (@acc ($e:expr, $(#[$attr:meta])* $raw:path => $type:ident, $($match_arms:tt)*) -> ($($body:tt)*)) => {
        $crate::enumutil::impl_enum_to!(@acc ($e, $($match_arms)*) -> ($($body)* $(#[$attr])* Self::$type => $raw,))
    };
    (@final ($e:expr) -> ($($body:tt)*)) => {
        match $e { $($body)* }
    };
    ($e:expr, $($match_arms:tt)*) => {
        $crate::enumutil::impl_enum_to!(@acc ($e, $($match_arms)*) -> ())
    };
}

pub(crate) use impl_enum;
pub(crate) use impl_enum_display;
pub(crate) use impl_enum_from;
pub(crate) use impl_enum_to;

#[cfg(test)]
mod tests {
    use super::*;

    const FOO: u32 = 0;
    const BAR: u32 = 1;
    const BAZ: u32 = 2;

    #[derive(Debug, PartialEq, Clone, Copy)]
    enum Example {
        Foo,
        Bar,
        Baz,
    }

    type ExampleEnum = Enum<Example, u32>;

    impl_enum! {
        enum: Example,
        raw: u32,
        match: {
            FOO => Foo,
            BAR => Bar,
            BAZ => Baz,
        }
    }

    #[test]
    fn test_enum_from_raw() {
        let inputs = [
            (FOO, Some(Example::Foo)),
            (BAR, Some(Example::Bar)),
            (BAZ, Some(Example::Baz)),
            (10, None),
        ];

        for &(raw, expected) in inputs.iter() {
            assert_eq!(Example::from_raw(raw), expected);
        }
    }

    #[test]
    fn test_enum_to_raw() {
        let inputs = [
            (Example::Foo, FOO, "foo"),
            (Example::Bar, BAR, "bar"),
            (Example::Baz, BAZ, "baz"),
        ];

        for &(variant, expected, estr) in inputs.iter() {
            assert_eq!(variant.to_raw(), expected);
            assert_eq!(variant.to_string(), estr);
        }
    }

    #[test]
    fn test_enum_known() {
        let e = ExampleEnum::from_raw(BAR);

        assert_eq!(e.to_raw(), BAR);
        assert!(e.is_known());
        assert!(!e.is_unknown());

        assert!(e.is_known_and(|k| k == Example::Bar));
        assert!(!e.is_known_and(|k| k == Example::Foo));
        assert!(!e.is_unknown_and(|_| true));

        assert!(e.is(Example::Bar));
        assert!(!e.is(Example::Foo));
        assert!(e.is_any(vec![Example::Foo, Example::Bar]));
        assert!(!e.is_any(vec![Example::Foo, Example::Baz]));

        assert_eq!(e.known(), Some(Example::Bar));
        assert_eq!(e.unknown(), None);

        assert_eq!(e.unwrap_or(Example::Foo), Example::Bar);
        assert_eq!(e.unwrap(), Example::Bar);

        assert_eq!(e.to_string(), "bar");
    }

    #[test]
    fn test_enum_result_known() {
        let e = ExampleEnum::from_raw(BAR);
        let r: Result<Example, UnknownEnumError<u32>> = e.into();

        assert!(r.is_ok());

        assert_eq!(r.ok(), Some(Example::Bar));
        assert_eq!(r.err(), None);

        assert_eq!(r.unwrap_or(Example::Foo), Example::Bar);
        assert_eq!(r.unwrap(), Example::Bar);
    }

    #[test]
    fn test_enum_unknown() {
        let e = ExampleEnum::from_raw(8);

        assert_eq!(e.to_raw(), 8);
        assert!(!e.is_known());
        assert!(e.is_unknown());

        assert!(!e.is_known_and(|_| true));
        assert!(e.is_unknown_and(|u| u == 8));
        assert!(!e.is_unknown_and(|u| u == 9));

        assert!(!e.is(Example::Bar));
        assert!(!e.is_any(vec![Example::Foo, Example::Bar]));

        assert_eq!(e.known(), None);
        assert_eq!(e.unknown(), Some(8));

        assert_eq!(e.unwrap_or(Example::Foo), Example::Foo);
        // type_name() output format is not guaranteed stable,
        // so merely check that our unqualified typename is contained
        // along with our value suffix
        assert!(e.to_string().contains("Example"));
        assert!(e.to_string().contains("(8)"));
    }

    #[test]
    fn test_enum_result_unknown() {
        let e = ExampleEnum::from_raw(8);
        let r: Result<Example, UnknownEnumError<u32>> = e.into();

        assert!(r.is_err());

        assert_eq!(r.ok(), None);
        assert_eq!(r.err(), Some(UnknownEnumError { value: 8 }));

        assert_eq!(r.unwrap_or(Example::Foo), Example::Foo);
    }
}
