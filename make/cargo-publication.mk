# Opt-in publication mechanics; consumers declare eligibility, registry and gates.
# Shared companions: make/execution.mk scripts/ci/cargo-publication-adapter.sh
_shared_publication_default_goal := $(.DEFAULT_GOAL)
include $(dir $(lastword $(MAKEFILE_LIST)))execution.mk
CARGO_PUBLISH_EVIDENCE_ROOT ?= $(CURDIR)/target/publication
_shared_publication_quote = '$(subst ','"'"',$(1))'

.PHONY: publish publish-check
publish publish-check:
	+@CARGO_PUBLISH_PACKAGES=$(call _shared_publication_quote,$(CARGO_PUBLISH_PACKAGES)) \
		CARGO_PUBLISH_SELECTION=$(call _shared_publication_quote,$(if $(filter command line,$(origin PACKAGE)),$(PACKAGE),$(CARGO_PUBLISH_PACKAGES))) \
		CARGO_PUBLISH_REGISTRY=$(call _shared_publication_quote,$(CARGO_PUBLISH_REGISTRY)) \
		CARGO_PUBLISH_VERIFY_TARGETS=$(call _shared_publication_quote,$(CARGO_PUBLISH_VERIFY_TARGETS)) \
		CARGO_PUBLISH_EVIDENCE_ROOT=$(call _shared_publication_quote,$(CARGO_PUBLISH_EVIDENCE_ROOT)) \
		CARGO_PUBLISH_MAKE=$(call _shared_publication_quote,$(MAKE_COMMAND)) \
		bash $(call _shared_publication_quote,$(SHARED_TOOLING_ROOT)/scripts/ci/cargo-publication-adapter.sh) $(if $(filter publish-check,$@),check,publish)

.DEFAULT_GOAL := $(_shared_publication_default_goal)
