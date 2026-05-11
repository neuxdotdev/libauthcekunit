# Makefile untuk libauthcekunit (Rust library dengan C FFI)

SHELL := /bin/bash
CARGO := cargo
CBINDGEN := cbindgen
CC := gcc
CFLAGS := -Wall -Wextra -O2 -g
LDFLAGS := -lpthread -ldl -lm

PROJECT := libauthcekunit
HEADER := $(PROJECT).h
TEST_SRC := tests.c
TEST_STATIC := test_static
TEST_DYNAMIC := test_dynamic

TARGET_DIR := target/release
STATIC_LIB := $(TARGET_DIR)/lib$(PROJECT).a
SHARED_LIB := $(TARGET_DIR)/lib$(PROJECT).so

INSTALL_PREFIX ?= /usr/local
INSTALL_LIBDIR := $(INSTALL_PREFIX)/lib
INSTALL_INCDIR := $(INSTALL_PREFIX)/include

.PHONY: all clean install uninstall test test-dyn help static dynamic header

all: static dynamic header

static: $(STATIC_LIB)
dynamic: $(SHARED_LIB)
header: $(HEADER)

$(STATIC_LIB) $(SHARED_LIB):
	$(CARGO) build --release

$(HEADER): src/lib.rs src/ffi.rs
	$(CBINDGEN) --lang c --output $(HEADER) .

test-static: $(STATIC_LIB) $(HEADER) $(TEST_SRC)
	$(CC) $(CFLAGS) -o $(TEST_STATIC) $(TEST_SRC) $(STATIC_LIB) $(LDFLAGS)

test-dynamic: $(SHARED_LIB) $(HEADER) $(TEST_SRC)
	$(CC) $(CFLAGS) -o $(TEST_DYNAMIC) $(TEST_SRC) -L$(TARGET_DIR) -l$(PROJECT) $(LDFLAGS)

test: test-static
	set -a; [ -f .env ] && . ./.env; set +a; ./$(TEST_STATIC)

test-dyn: test-dynamic
	set -a; [ -f .env ] && . ./.env; set +a; LD_LIBRARY_PATH=$(TARGET_DIR) ./$(TEST_DYNAMIC)

install: all
	install -Dm755 $(SHARED_LIB) $(INSTALL_LIBDIR)/lib$(PROJECT).so
	install -Dm644 $(STATIC_LIB) $(INSTALL_LIBDIR)/lib$(PROJECT).a
	install -Dm644 $(HEADER) $(INSTALL_INCDIR)/$(PROJECT).h

uninstall:
	rm -f $(INSTALL_LIBDIR)/lib$(PROJECT).so
	rm -f $(INSTALL_LIBDIR)/lib$(PROJECT).a
	rm -f $(INSTALL_INCDIR)/$(PROJECT).h

clean:
	$(CARGO) clean
	rm -f $(HEADER) $(TEST_STATIC) $(TEST_DYNAMIC)

help:
	@echo "Target:"
	@echo "  all          : Build static library, shared library, and header"
	@echo "  static       : Build static library (.a)"
	@echo "  dynamic      : Build shared library (.so)"
	@echo "  header       : Generate C header using cbindgen"
	@echo "  test         : Build and run static test (loads .env if present)"
	@echo "  test-dyn     : Build and run dynamic test (loads .env)"
	@echo "  install      : Install libraries and header to $(INSTALL_PREFIX)"
	@echo "  uninstall    : Remove installed files"
	@echo "  clean        : Remove build artifacts"
	@echo "  help         : Show this help"
	@echo ""
	@echo "Variables:"
	@echo "  INSTALL_PREFIX = $(INSTALL_PREFIX)"
	@echo "  CC            = $(CC)"
	@echo "  CFLAGS        = $(CFLAGS)"