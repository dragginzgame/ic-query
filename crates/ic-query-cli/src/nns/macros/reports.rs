macro_rules! impl_nns_leaf_reports {
    (
        $reports:ident,
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
            type ListReport = $list_report;
            type InfoReport = $info_report;
            type RefreshReport = $refresh_report;
            type HostError = $host_error;

            fn build_list_report(
                &self,
                request: &ic_query::nns::NnsInventoryListRequest,
                progress: &mut dyn ic_query::QueryProgress,
            ) -> Result<Self::ListReport, Self::HostError> {
                $build_list(request, progress)
            }

            fn build_info_report(
                &self,
                request: &ic_query::nns::NnsInventoryInfoRequest,
                progress: &mut dyn ic_query::QueryProgress,
            ) -> Result<Self::InfoReport, Self::HostError> {
                $build_info(request, progress)
            }

            fn refresh_report(
                &self,
                request: &ic_query::nns::NnsInventoryRefreshRequest,
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
