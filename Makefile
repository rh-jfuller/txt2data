.PHONY: all build release check test clippy fmt clean install wasm serve
.PHONY: examples example-greeting example-csv example-keyvalue
.PHONY: example-date example-http example-attribute

CARGO = cargo
BIN = txt2data

# --- Cargo targets ---

all: check

build:
	$(CARGO) build

release:
	$(CARGO) build --release

check: fmt clippy test

test:
	$(CARGO) test

clippy:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

fmt:
	$(CARGO) fmt --check

fmt-fix:
	$(CARGO) fmt

clean:
	$(CARGO) clean

install:
	$(CARGO) install --path .

wasm:
	wasm-pack build --target web --features wasm --no-default-features

serve: wasm
	ln -sfn ../pkg etc/pkg
	@echo "Open http://localhost:8080"
	python3 -m http.server 8080 -d etc

# --- Examples ---

examples: example-greeting example-csv example-keyvalue \
          example-date example-http example-attribute

example-greeting:
	@echo "=== greeting (json) ==="
	$(CARGO) run -q -- -g etc/greeting.ixml -i etc/greeting.txt
	@echo ""
	@echo "=== greeting (xml) ==="
	$(CARGO) run -q -- -g etc/greeting.ixml -i etc/greeting.txt -f xml
	@echo ""

example-csv:
	@echo "=== csv (json) ==="
	$(CARGO) run -q -- -g etc/csv.ixml -i etc/csv.txt
	@echo ""
	@echo "=== csv (xml) ==="
	$(CARGO) run -q -- -g etc/csv.ixml -i etc/csv.txt -f xml
	@echo ""

example-keyvalue:
	@echo "=== keyvalue (json) ==="
	$(CARGO) run -q -- -g etc/keyvalue.ixml -i etc/keyvalue.txt
	@echo ""
	@echo "=== keyvalue (xml) ==="
	$(CARGO) run -q -- -g etc/keyvalue.ixml -i etc/keyvalue.txt -f xml
	@echo ""

example-date:
	@echo "=== date (json) ==="
	$(CARGO) run -q -- -g etc/date.ixml -i etc/date.txt
	@echo ""
	@echo "=== date (xml) ==="
	$(CARGO) run -q -- -g etc/date.ixml -i etc/date.txt -f xml
	@echo ""

example-http:
	@echo "=== http request (json) ==="
	$(CARGO) run -q -- -g etc/http.ixml -i etc/http.txt
	@echo ""
	@echo "=== http request (xml) ==="
	$(CARGO) run -q -- -g etc/http.ixml -i etc/http.txt -f xml
	@echo ""

example-attribute:
	@echo "=== attribute marks (json) ==="
	$(CARGO) run -q -- -g etc/attribute.ixml -i etc/attribute.txt
	@echo ""
	@echo "=== attribute marks (xml) ==="
	$(CARGO) run -q -- -g etc/attribute.ixml -i etc/attribute.txt -f xml
	@echo ""
