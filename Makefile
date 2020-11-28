CPPFLAGS := -D_POSIX_C_SOURCE=200809L
CFLAGS   := -std=c99 -Wall -Wextra

BINS := \
    pista-sensor-battery \
    pista-sensor-time

.PHONY: build clean rebuild

build: $(BINS)

pista-sensor-battery: \
	pista_log.o \
	pista_time.o

pista-sensor-time: \
	pista_log.o \
	pista_time.o

pista_time.o: pista_log.o

clean:
	rm -f *.o
	rm -f $(BINS)

rebuild:
	@$(MAKE) -s clean
	@$(MAKE) -s build
