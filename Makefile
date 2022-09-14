N_CPUS    := $(shell nproc 2> /dev/null || gnproc 2> /dev/null || sysctl -n hw.ncpu 2> /dev/null)
MAKEFLAGS := -j $(N_CPUS) --no-print-directory
CPPFLAGS := -D_POSIX_C_SOURCE=200809L
CFLAGS   := -std=c99 -Wall -Wextra

BINS := \
    pista-sensor-backlight-laptop \
    pista-sensor-battery \
    pista-sensor-time \
    pista-sensor-upower \
    pista-sensor-memory \
    pista-sensor-mpd \
    pista-sensor-mpd-rkt \
    pista-sensor-openweather \
    pista-sensor-weather-gov \
    pista-sensor-helium-account-balance

.PHONY: build clean_all clean_bins clean_deps rebuild install reinstall deps

build: $(BINS)

.PHONY: test
test:
	raco test ./*.rkt
	cargo test

pista-sensor-battery: \
	pista_log.o \
	pista_time.o

pista-sensor-time: \
	pista_log.o \
	pista_time.o

pista-sensor-backlight-laptop: | rust
	mv target/release/$@ ./

pista-sensor-helium-account-balance: | rust
	mv target/release/$@ ./

pista-sensor-memory: | rust
	mv target/release/$@ ./

pista-sensor-mpd: | rust
	mv target/release/$@ ./

.PHONY: rust
rust:
	cargo build --release

pista-sensor-mpd-rkt: pista-sensor-mpd-rkt.rkt
	raco exe --orig-exe -o $@ $<

pista-sensor-openweather: pista-sensor-openweather.rkt
	raco exe --orig-exe -o $@ $<

pista-sensor-upower: pista-sensor-upower.rkt
	raco exe --orig-exe -o $@ $<

pista-sensor-weather-gov: pista-sensor-weather-gov.rkt
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
	find . -type f -name 'pista-sensor-*' -executable -exec cp -f '{}' ~/bin/ \;

install_init:
	cp ./example-via-tmux ~/.xinitrc-pista

reinstall:
	raco pkg remove pista-sensors || true

deps:
	raco pkg install --deps search-auto
