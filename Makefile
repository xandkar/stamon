N_CPUS    := $(shell nproc 2> /dev/null || gnproc 2> /dev/null || sysctl -n hw.ncpu 2> /dev/null)
MAKEFLAGS := -j $(N_CPUS)
CPPFLAGS := -D_POSIX_C_SOURCE=200809L
CFLAGS   := -std=c99 -Wall -Wextra

BINS := \
    pista-sensor-battery \
    pista-sensor-time \
    pista-sensor-upower \
    pista-sensor-mpd \
    pista-sensor-openweather \
    pista-sensor-weather-gov \
    pista-sensor-helium-account-balance-rs

.PHONY: build clean rebuild install reinstall deps

build: $(BINS)

.PHONY: test
test:
	raco test ./pista-sensor-mpd.rkt

pista-sensor-battery: \
	pista_log.o \
	pista_time.o

pista-sensor-time: \
	pista_log.o \
	pista_time.o

%-rs:
	cd $@.src && cargo build && mv target/debug/$@ ../

pista-sensor-mpd: pista-sensor-mpd.rkt
	raco exe --orig-exe -o $@ $<

pista-sensor-openweather: pista-sensor-openweather.rkt
	raco exe --orig-exe -o $@ $<

pista-sensor-upower: pista-sensor-upower.rkt
	raco exe --orig-exe -o $@ $<

pista-sensor-weather-gov: pista-sensor-weather-gov.rkt
	raco exe --orig-exe -o $@ $<

pista_time.o: pista_log.o

clean:
	rm -f *.o
	rm -f $(BINS)
	rm -rf compiled/
	rm -rf *-rs.src/target/

rebuild:
	@$(MAKE) -s clean
	@$(MAKE) -s build

install:
	mkdir -p ~/bin
	find . -type f -name 'pista-sensor-*' -executable -exec cp -f '{}' ~/bin/ \;

install_init:
	cp ./example-via-tmux ~/.xinitrc-pista

reinstall:
	raco pkg remove pista-sensors || true

deps:
	raco pkg install --deps search-auto
