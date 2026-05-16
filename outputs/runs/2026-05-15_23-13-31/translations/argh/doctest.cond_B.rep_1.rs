mod doctest {
    pub const DOCTEST_VERSION_MAJOR: i32 = 2;
    pub const DOCTEST_VERSION_MINOR: i32 = 4;
    pub const DOCTEST_VERSION_PATCH: i32 = 6;
    pub const DOCTEST_VERSION_STR: &str = "2.4.6";

    pub const DOCTEST_VERSION: i32 = DOCTEST_VERSION_MAJOR * 10000 +
                                      DOCTEST_VERSION_MINOR * 100 +
                                      DOCTEST_VERSION_PATCH;

    #[cfg(target_os = "windows")]
    mod platform {
        pub const DOCTEST_PLATFORM_WINDOWS: bool = true;
    }

    #[cfg(target_os = "linux")]
    mod platform {
        pub const DOCTEST_PLATFORM_LINUX: bool = true;
    }

    #[cfg(target_os = "macos")]
    mod platform {
        pub const DOCTEST_PLATFORM_MAC: bool = true;
    }

    #[cfg(target_os = "ios")]
    mod platform {
        pub const DOCTEST_PLATFORM_IPHONE: bool = true;
    }

    mod color {
        #[derive(PartialEq, Clone, Copy)]
        pub enum Color {
            None = 0,
            White,
            Red,
            Green,
            Blue,
            Cyan,
            Yellow,
            Grey,
            Bright = 0x10,
            BrightRed = Self::Bright as isize | Self::Red as isize,
            BrightGreen = Self::Bright as isize | Self::Green as isize,
            LightGrey = Self::Bright as isize | Self::Grey as isize,
            BrightWhite = Self::Bright as isize | Self::White as isize,
        }
    }

    #[derive(PartialEq, Clone, Copy)]
    pub enum AssertType {
        IsWarn = 1,
        IsCheck = 2 * 1,
        IsRequire = 2 * 2,
        IsThrows = 2 * 4,
        IsThrowsAs = 2 * 8,
        IsThrowsWith = 2 * 16,
        IsNothrow = 2 * 32,
        IsFalse = 2 * 64,
        IsUnary = 2 * 128,
        IsEq = 2 * 256,
        IsNe = 2 * 512,
        IsLt = 2 * 1024,
        IsGt = 2 * 2048,
        IsGe = 2 * 4096,
        IsLe = 2 * 8192,
    }

    use std::ops::BitOr;

    impl BitOr for AssertType {
        type Output = i32;

        fn bitor(self, rhs: Self) -> Self::Output {
            self as i32 | rhs as i32
        }
    }

    mod assert_type {
        use super::AssertType;

        impl AssertType {
            pub const DT_WARN: i32 = AssertType::IsWarn as i32 | AssertType::IsWarn as i32;
            pub const DT_CHECK: i32 = AssertType::IsCheck as i32 | AssertType::IsCheck as i32;
            pub const DT_REQUIRE: i32 = AssertType::IsRequire as i32 | AssertType::IsRequire as i32;

            pub const DT_WARN_FALSE: i32 = AssertType::IsWarn as i32 | AssertType::IsFalse as i32 | AssertType::IsWarn as i32;
            pub const DT_CHECK_FALSE: i32 = AssertType::IsCheck as i32 | AssertType::IsFalse as i32 | AssertType::IsCheck as i32;
            pub const DT_REQUIRE_FALSE: i32 = AssertType::IsRequire as i32 | AssertType::IsFalse as i32 | AssertType::IsRequire as i32;

            pub const DT_WARN_THROWS: i32 = AssertType::IsThrows as i32 | AssertType::IsWarn as i32;
            pub const DT_CHECK_THROWS: i32 = AssertType::IsThrows as i32 | AssertType::IsCheck as i32;
            pub const DT_REQUIRE_THROWS: i32 = AssertType::IsThrows as i32 | AssertType::IsRequire as i32;

            pub const DT_WARN_EQ: i32 = AssertType::IsWarn as i32 | AssertType::IsEq as i32 | AssertType::IsWarn as i32;
            pub const DT_CHECK_EQ: i32 = AssertType::IsCheck as i32 | AssertType::IsEq as i32 | AssertType::IsCheck as i32;
            pub const DT_REQUIRE_EQ: i32 = AssertType::IsRequire as i32 | AssertType::IsEq as i32 | AssertType::IsRequire as i32;

            pub const DT_WARN_NE: i32 = AssertType::IsWarn as i32 | AssertType::IsNe as i32 | AssertType::IsWarn as i32;
            pub const DT_CHECK_NE: i32 = AssertType::IsCheck as i32 | AssertType::IsNe as i32 | AssertType::IsCheck as i32;
            pub const DT_REQUIRE_NE: i32 = AssertType::IsRequire as i32 | AssertType::IsNe as i32 | AssertType::IsRequire as i32;

            pub const DT_WARN_GT: i32 = AssertType::IsWarn as i32 | AssertType::IsGt as i32 | AssertType::IsWarn as i32;
            pub const DT_CHECK_GT: i32 = AssertType::IsCheck as i32 | AssertType::IsGt as i32 | AssertType::IsCheck as i32;
            pub const DT_REQUIRE_GT: i32 = AssertType::IsRequire as i32 | AssertType::IsGt as i32 | AssertType::IsRequire as i32;

            pub const DT_WARN_LT: i32 = AssertType::IsWarn as i32 | AssertType::IsLt as i32 | AssertType::IsWarn as i32;
            pub const DT_CHECK_LT: i32 = AssertType::IsCheck as i32 | AssertType::IsLt as i32 | AssertType::IsCheck as i32;
            pub const DT_REQUIRE_LT: i32 = AssertType::IsRequire as i32 | AssertType::IsLt as i32 | AssertType::IsRequire as i32;

            pub const DT_WARN_GE: i32 = AssertType::IsWarn as i32 | AssertType::IsGe as i32 | AssertType::IsWarn as i32;
            pub const DT_CHECK_GE: i32 = AssertType::IsCheck as i32 | AssertType::IsGe as i32 | AssertType::IsCheck as i32;
            pub const DT_REQUIRE_GE: i32 = AssertType::IsRequire as i32 | AssertType::IsGe as i32 | AssertType::IsRequire as i32;

            pub const DT_WARN_LE: i32 = AssertType::IsWarn as i32 | AssertType::IsLe as i32 | AssertType::IsWarn as i32;
            pub const DT_CHECK_LE: i32 = AssertType::IsCheck as i32 | AssertType::IsLe as i32 | AssertType::IsCheck as i32;
            pub const DT_REQUIRE_LE: i32 = AssertType::IsRequire as i32 | AssertType::IsLe as i32 | AssertType::IsRequire as i32;

            pub const DT_WARN_UNARY: i32 = AssertType::IsWarn as i32 | AssertType::IsUnary as i32 | AssertType::IsWarn as i32;
            pub const DT_CHECK_UNARY: i32 = AssertType::IsCheck as i32 | AssertType::IsUnary as i32 | AssertType::IsCheck as i32;
            pub const DT_REQUIRE_UNARY: i32 = AssertType::IsRequire as i32 | AssertType::IsUnary as i32 | AssertType::IsRequire as i32;

            pub const DT_WARN_UNARY_FALSE: i32 = AssertType::IsWarn as i32 | AssertType::IsFalse as i32 | AssertType::IsUnary as i32 | AssertType::IsWarn as i32;
            pub const DT_CHECK_UNARY_FALSE: i32 = AssertType::IsCheck as i32 | AssertType::IsFalse as i32 | AssertType::IsUnary as i32 | AssertType::IsCheck as i32;
            pub const DT_REQUIRE_UNARY_FALSE: i32 = AssertType::IsRequire as i32 | AssertType::IsFalse as i32 | AssertType::IsUnary as i32 | AssertType::IsRequire as i32;
        }
    }
}

fn main() {
    // Entry point for the application
}