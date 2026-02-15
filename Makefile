VERSION := $(shell cat version.txt | sed 's/^v//')

version:
	@echo v$(VERSION)

publish-patch:
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. -v OFS=. '{$$NF++;print}'); \
	echo v$$NEW_VERSION > version.txt; \
	git add version.txt; \
	git commit -m "Bump version to v$$NEW_VERSION"; \
	git tag v$$NEW_VERSION; \
	git push; \
	git push origin v$$NEW_VERSION

publish-minor:
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. -v OFS=. '{$$2++;$$3=0;print}'); \
	echo v$$NEW_VERSION > version.txt; \
	git add version.txt; \
	git commit -m "Bump version to v$$NEW_VERSION"; \
	git tag v$$NEW_VERSION; \
	git push; \
	git push origin v$$NEW_VERSION

publish-major:
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. -v OFS=. '{$$1++;$$2=0;$$3=0;print}'); \
	echo v$$NEW_VERSION > version.txt; \
	git add version.txt; \
	git commit -m "Bump version to v$$NEW_VERSION"; \
	git tag v$$NEW_VERSION; \
	git push; \
	git push origin v$$NEW_VERSION
