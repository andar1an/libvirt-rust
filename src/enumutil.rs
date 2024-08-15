pub trait RawEnum<I>: Sized {
    fn from_raw(raw: I) -> Option<Self>;
    fn to_raw(self) -> I;
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
}
