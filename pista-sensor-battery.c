#include <signal.h>
#include <assert.h>
#include <ctype.h>
#include <errno.h>
#include <limits.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include "pista_log.h"
#include "pista_time.h"

#define usage(...) {print_usage(); fprintf(stderr, "Error:\n    " __VA_ARGS__); exit(EXIT_FAILURE);}

#define MAX_LEN 20
#define PREFIX "⚡"
#define POSTFIX "%"
#define BUF_SIZE sizeof(PREFIX) + sizeof("100") + sizeof(POSTFIX) + sizeof(NULL)

char *argv0;

double  opt_interval = 1.0;
char   *opt_battery  = "BAT0";

void
print_usage()
{
	printf(
	    "%s: [OPT ...]\n"
	    "\n"
	    "OPT = -i int     # interval\n"
	    "    | -b string  # battery file name from /sys/class/power_supply/\n"
	    "    | -h         # help message (i.e. what you're reading now :) )\n",
	    argv0);
}

void
opt_parse(int argc, char **argv)
{
	char c;

	while ((c = getopt(argc, argv, "b:i:h")) != -1)
		switch (c) {
		case 'b':
			opt_battery = calloc(strlen(optarg), sizeof(char));
			strcpy(opt_battery, optarg);
			break;
		case 'i':
			opt_interval = atof(optarg);
			break;
		case 'h':
			print_usage();
			exit(EXIT_SUCCESS);
		case '?':
			if (optopt == 'b' || optopt == 'i')
				fprintf(stderr,
					"Option -%c requires an argument.\n",
					optopt);
			else if (isprint(optopt))
				fprintf (stderr,
					"Unknown option `-%c'.\n",
					optopt);
			else
				fprintf(stderr,
					"Unknown option character `\\x%x'.\n",
					optopt);
			exit(EXIT_FAILURE);
		default:
			assert(0);
		}
}

int
get_capacity(char *buf, char *path)
{
	FILE *fp;
	int cap;

	if (!(fp = fopen(path, "r")))
		pista_fatal(
		    "Failed to open %s. errno: %d, msg: %s\n",
		    path,
		    errno,
		    strerror(errno)
		);

	switch (fscanf(fp, "%d", &cap)) {
	case -1: pista_fatal("EOF\n");
	case  0: pista_fatal("Read 0\n");
	case  1: break;
	default: assert(0);
	}
	fclose(fp);
	return snprintf(buf, BUF_SIZE, "%s%3d%s", PREFIX, cap, POSTFIX);
}

int
main(int argc, char **argv)
{
	/* TODO support multiple batteries */
	/* TODO track and show (de|in)crease */
	argv0 = argv[0];

	char buf[BUF_SIZE];
	char path[PATH_MAX];
	char *path_fmt = "/sys/class/power_supply/%s/capacity";
	struct timespec ti;

	opt_parse(argc, argv);
	ti = pista_timespec_of_float(opt_interval);
	pista_debug("buf size: %lu\n", BUF_SIZE);
	pista_debug("opt_battery: \"%s\"\n", opt_battery);
	pista_debug("opt_interval: %f\n", opt_interval);
	pista_debug("ti: {tv_sec = %ld, tv_nsec = %ld}\n",ti.tv_sec,ti.tv_nsec);
	memset(path, '\0', PATH_MAX);
	snprintf(path, PATH_MAX, path_fmt, opt_battery);
	signal(SIGPIPE, SIG_IGN);

	for (;;) {
		get_capacity(buf, path);
		puts(buf);
		fflush(stdout);
		pista_sleep(&ti);
	}
	return EXIT_SUCCESS;
}
