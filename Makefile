VERSION := $(shell awk '/^\[package\]/{pkg=1;next} /^\[/{pkg=0} pkg && /^version = /{gsub(/"/,"",$$3); print $$3; exit}' Cargo.toml)
UPDATE_VERSION_CMD = bash -c 'set -euo pipefail; new="$$NEW_VERSION"; tmp="$$(mktemp)"; \
awk -v new="$$new" '\''BEGIN{inpkg=0; updated=0} \
  /^\[package\]/{inpkg=1} \
  /^\[/{if ($$0 !~ /^\[package\]/) inpkg=0} \
  inpkg && /^version[[:space:]]*=/ {sub(/"[^"]*"/, "\"" new "\""); updated=1} \
  {print} \
  END{if(!updated){exit 2}}'\'' Cargo.toml > "$$tmp"; \
mv "$$tmp" Cargo.toml'

version:
	@echo v$(VERSION)

publish-patch:
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. -v OFS=. '{$$NF++;print}'); \
	$(UPDATE_VERSION_CMD); \
	git add Cargo.toml; \
	git commit -m "Bump version to v$$NEW_VERSION"; \
	git tag v$$NEW_VERSION; \
	git push; \
	git push origin v$$NEW_VERSION

publish-minor:
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. -v OFS=. '{$$2++;$$3=0;print}'); \
	$(UPDATE_VERSION_CMD); \
	git add Cargo.toml; \
	git commit -m "Bump version to v$$NEW_VERSION"; \
	git tag v$$NEW_VERSION; \
	git push; \
	git push origin v$$NEW_VERSION

publish-major:
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. -v OFS=. '{$$1++;$$2=0;$$3=0;print}'); \
	$(UPDATE_VERSION_CMD); \
	git add Cargo.toml; \
	git commit -m "Bump version to v$$NEW_VERSION"; \
	git tag v$$NEW_VERSION; \
	git push; \
	git push origin v$$NEW_VERSION
