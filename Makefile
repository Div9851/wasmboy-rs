.PHONY: build watch

BUILD=wasm-pack build core --target web

build:
	$(BUILD)

watch:
	find core/src/ | entr -r $(BUILD)
