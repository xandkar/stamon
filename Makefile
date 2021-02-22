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
    pista-sensor-weather-gov

.PHONY: build clean rebuild install reinstall

build: $(BINS)

pista-sensor-battery: \
	pista_log.o \
	pista_time.o

pista-sensor-time: \
	pista_log.o \
	pista_time.o

pista-sensor-mpd: pista-sensor-mpd.rkt
	raco exe -o $@ $<

pista-sensor-openweather: pista-sensor-openweather.rkt
	raco exe -o $@ $<

pista-sensor-upower: pista-sensor-upower.rkt
	raco exe -o $@ $<

pista-sensor-weather-gov: pista-sensor-weather-gov.rkt
	raco exe -o $@ $<

pista_time.o: pista_log.o

clean:
	rm -f *.o
	rm -f $(BINS)

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
	raco pkg install --deps search-auto
