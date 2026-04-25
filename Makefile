.PHONY: publish
publish:
	cargo package --allow-dirty
	cargo package --list --allow-dirty
	cargo publish --dry-run --allow-dirty
