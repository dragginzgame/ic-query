macro_rules! impl_leaf_test_helpers {
    (
        $list_options:ident,
        $info_options:ident,
        $refresh_options:ident,
        $usage:ident,
        $list_usage:ident,
        $info_usage:ident,
        $refresh_usage:ident,
        $spec:ident,
        $default_source_endpoint:expr
    ) => {
        pub(in crate::nns) fn $list_options<I>(
            args: I,
        ) -> Result<leaf::NnsLeafListOptions, NnsCommandError>
        where
            I: IntoIterator<Item = std::ffi::OsString>,
        {
            leaf::NnsLeafListOptions::parse(args, &$spec, $default_source_endpoint)
        }

        pub(in crate::nns) fn $info_options<I>(
            args: I,
        ) -> Result<leaf::NnsLeafInfoOptions, NnsCommandError>
        where
            I: IntoIterator<Item = std::ffi::OsString>,
        {
            leaf::NnsLeafInfoOptions::parse(args, &$spec, $default_source_endpoint)
        }

        pub(in crate::nns) fn $refresh_options<I>(
            args: I,
        ) -> Result<leaf::NnsLeafRefreshOptions, NnsCommandError>
        where
            I: IntoIterator<Item = std::ffi::OsString>,
        {
            leaf::NnsLeafRefreshOptions::parse(args, &$spec, $default_source_endpoint)
        }

        pub(in crate::nns) fn $usage() -> String {
            leaf::usage(&$spec)
        }

        pub(in crate::nns) fn $list_usage() -> String {
            leaf::list_usage(&$spec, $default_source_endpoint)
        }

        pub(in crate::nns) fn $info_usage() -> String {
            leaf::info_usage(&$spec, $default_source_endpoint)
        }

        pub(in crate::nns) fn $refresh_usage() -> String {
            leaf::refresh_usage(&$spec, $default_source_endpoint)
        }
    };
}
