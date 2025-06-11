test:
	@echo "Running Helm chart validation tests..."
	cd tests && cargo test
	@echo "✅ Helm chart tests passed!"

test-life:
	@echo "Running Life Helm chart tests..."
	cd tests && cargo test --test life_chart_tests
	@echo "✅ Life Helm chart tests passed!"

test-foundry:
	@echo "Foundry chart tests not implemented yet"

.PHONY: test test-life test-foundry