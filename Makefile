N_CPUS    := $(shell nproc 2> /dev/null || gnproc 2> /dev/null || sysctl -n hw.ncpu 2> /dev/null)
MAKEFLAGS := -j $(N_CPUS) --no-print-directory
CPPFLAGS := -D_POSIX_C_SOURCE=200809L
CFLAGS   := -std=c99 -Wall -Wextra

BINS := \
    pista-feed-backlight-laptop \
    pista-feed-battery \
    pista-feed-disk \
    pista-feed-time \
    pista-feed-upower \
    pista-feed-memory \
    pista-feed-mpd \
    pista-feed-net \
    pista-feed-volume \
    pista-feed-weather \
    pista-feed-helium-account-balance \
    pista-feed-x11-keymap \


.PHONY: build clean_all clean_bins clean_deps rebuild install reinstall deps

build: $(BINS)

.PHONY: test
test:
	raco test ./*.rkt
	cargo test

pista-feed-battery: \
	pista_log.o \
	pista_time.o

pista-feed-time: \
	pista_log.o \
	pista_time.o

pista-feed-backlight-laptop: | rust
	mv target/release/$@ ./

pista-feed-disk: | rust
	mv target/release/$@ ./

pista-feed-helium-account-balance: | rust
	mv target/release/$@ ./

pista-feed-memory: | rust
	mv target/release/$@ ./

pista-feed-mpd: | rust
	mv target/release/$@ ./

pista-feed-net: | rust
	mv target/release/$@ ./

pista-feed-volume: | rust
	mv target/release/$@ ./

pista-feed-weather: | rust
	mv target/release/$@ ./

pista-feed-x11-keymap: | rust
	mv target/release/$@ ./

.PHONY: rust
rust:
	cargo build --release

pista-feed-upower: pista-feed-upower.rkt
	raco exe --orig-exe -o $@ $<

pista_time.o: pista_log.o

clean_all: clean_bins clean_deps

clean_bins:
	rm -f *.o
	rm -f $(BINS)

clean_deps:
	rm -rf compiled/
	rm -rf target/

rebuild:
	$(MAKE) clean_bins
	$(MAKE) build

install:
	mkdir -p ~/bin
	find . -type f -name 'pista-feed-*' -executable -exec cp -f '{}' ~/bin/ \;

install_init:
	cp ./example-via-tmux ~/.xinitrc-pista

reinstall:
	raco pkg remove pista-feeds || true

deps:
	raco pkg install --deps search-auto
