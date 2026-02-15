VERSION=0.6.0

version:
	@echo v$(VERSION)

publish-patch:
	@$(eval NEW_VERSION=$(shell echo $(VERSION) | awk -F. -v OFS=. '{$NF++;print}'))
	@sed -i '' 's/^VERSION=.*/VERSION=$(NEW_VERSION)/' Makefile
	@git add Makefile
	@git commit -m "Bump version to v$(NEW_VERSION)"
	@git tag v$(NEW_VERSION)
	@git push
	@git push origin v$(NEW_VERSION)

publish-minor:
	@$(eval NEW_VERSION=$(shell echo $(VERSION) | awk -F. -v OFS=. '{$2++;$3=0;print}'))
	@sed -i '' 's/^VERSION=.*/VERSION=$(NEW_VERSION)/' Makefile
	@git add Makefile
	@git commit -m "Bump version to v$(NEW_VERSION)"
	@git tag v$(NEW_VERSION)
	@git push
	@git push origin v$(NEW_VERSION)

publish-major:
	@$(eval NEW_VERSION=$(shell echo $(VERSION) | awk -F. -v OFS=. '{$1++;$2=0;$3=0;print}'))
	@sed -i '' 's/^VERSION=.*/VERSION=$(NEW_VERSION)/' Makefile
	@git add Makefile
	@git commit -m "Bump version to v$(NEW_VERSION)"
	@git tag v$(NEW_VERSION)
	@git push
	@git push origin v$(NEW_VERSION)
