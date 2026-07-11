macro_rules! impl_nns_leaf_reports {
    (
        $reports:ident,
        cache = $cache:ty,
        list_request = $list_request:ty,
        info_request = $info_request:ty,
        refresh_request = $refresh_request:ty,
        list_report = $list_report:ty,
        info_report = $info_report:ty,
        refresh_report = $refresh_report:ty,
        host_error = $host_error:ty,
        build_list = $build_list:ident,
        build_info = $build_info:ident,
        refresh = $refresh:ident,
        list_text = $list_text:ident,
        list_verbose_text = $list_verbose_text:ident,
        info_text = $info_text:ident,
        refresh_text = $refresh_text:ident $(,)?
    ) => {
        pub(super) struct $reports;

        impl leaf::NnsLeafReports for $reports {
            type Cache = $cache;
            type ListRequest = $list_request;
            type InfoRequest = $info_request;
            type RefreshRequest = $refresh_request;
            type ListReport = $list_report;
            type InfoReport = $info_report;
            type RefreshReport = $refresh_report;
            type HostError = $host_error;

            fn build_list_report(
                &self,
                request: &Self::ListRequest,
            ) -> Result<Self::ListReport, Self::HostError> {
                $build_list(request)
            }

            fn build_info_report(
                &self,
                request: &Self::InfoRequest,
            ) -> Result<Self::InfoReport, Self::HostError> {
                $build_info(request)
            }

            fn refresh_report(
                &self,
                request: &Self::RefreshRequest,
            ) -> Result<Self::RefreshReport, Self::HostError> {
                $refresh(request)
            }

            fn list_report_text(&self, report: &Self::ListReport) -> String {
                $list_text(report)
            }

            fn list_report_verbose_text(&self, report: &Self::ListReport) -> String {
                $list_verbose_text(report)
            }

            fn info_report_text(&self, report: &Self::InfoReport) -> String {
                $info_text(report)
            }

            fn refresh_report_text(&self, report: &Self::RefreshReport) -> String {
                $refresh_text(report)
            }
        }
    };
}
