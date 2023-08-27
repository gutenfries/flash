/*
 * A command line utility for reading OBD-II diagnostic data from a
 * vehicle via a CAN socket.
 *
 * This file is based off the file `isotprecv.c` contained in the
 * https://github.com/linux-can/can-utils GitHub repository.
 * The original copyright notice is reproduced below.
 */

/*
 * isotprecv.c
 *
 * Copyright (c) 2008 Volkswagen Group Electronic Research
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. Neither the name of Volkswagen nor the names of its contributors
 *    may be used to endorse or promote products derived from this software
 *    without specific prior written permission.
 *
 * Alternatively, provided that this notice is retained in full, this
 * software may be distributed under the terms of the GNU General
 * Public License ("GPL") version 2, in which case the provisions of the
 * GPL apply INSTEAD OF those given above.
 *
 * The provided data structures and external interfaces from this code
 * are not restricted to be used by modules with a GPL compatible license.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH
 * DAMAGE.
 *
 * Send feedback to <linux-can@vger.kernel.org>
 *
 */

#include <errno.h>
#include <getopt.h>
#include <libgen.h>
#include <linux/can.h>
#include <net/if.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

#include "OBDII.h"
#include "OBDIICommunication.h"

#define NO_CAN_ID 0xFFFFFFFFU
#define BUFSIZE 5000 /* size > 4095 to check socket API internal checks */

int interrupted = 0;

void print_usage(char* program_name) {
	printf("Usage: %s -t <transfer CAN ID> -r <receive CAN ID> [-d] <CAN "
		   "interface>\n	<transfer CAN ID>: The CAN ID that will be used for "
		   "sending the diagnostic requests. For 11-bit identifiers, this can be "
		   "either the broadcast ID, 0x7DF, or an ID in the range 0x7E0 to 0x7E7, "
		   "indicating a particular ECU.\n	<receive CAN ID>: The CAN ID that the "
		   "ECU will be using to respond to the diagnostic requests that are sent. "
		   "For 11-bit identifiers, this is an ID in the range 0x7E8 to 0x7EF (i.e. "
		   "<transfer CAN ID> + 8)\n	-d: Use a shared socket to allow other "
		   "programs to access the ECU (the obdiid daemon must be running for this "
		   "to work)\n",
		   program_name);
}

void handleInterrupted(int signum) {
	interrupted = 1;
}

int main(int argc, char** argv) {
	OBDIISocket s;
	int opt, i, use_daemon = 0;
	extern int optind, opterr, optopt;
	canid_t tx_id = NO_CAN_ID, rx_id = NO_CAN_ID;

	while ((opt = getopt(argc, argv, "r:t:d")) != -1) {
		switch (opt) {
			case 't':
				tx_id = strtoul(optarg, (char**)NULL, 16);
				if (strlen(optarg) > 7) {
					tx_id |= CAN_EFF_FLAG;
				}
				break;

			case 'r':
				rx_id = strtoul(optarg, (char**)NULL, 16);
				if (strlen(optarg) > 7) {
					rx_id |= CAN_EFF_FLAG;
				}
				break;
			case 'd':
				use_daemon = 1;
				break;

			default:
				fprintf(stderr, "Unknown option %c\n", opt);
				print_usage(basename(argv[0]));
				exit(1);
				break;
		}
	}

	if ((argc - optind != 1) || (tx_id == NO_CAN_ID) || (rx_id == NO_CAN_ID)) {
		print_usage(basename(argv[0]));
		exit(1);
	}

	//   // Install SIGINT handler
	//   struct sigaction interruptSignalAction;
	//   sigemptyset(&interruptSignalAction.sa_mask);
	//   interruptSignalAction.sa_flags = 0;
	//   interruptSignalAction.sa_handler = &handleInterrupted;

	//   sigaction(SIGINT, &interruptSignalAction, NULL);

	if ((OBDIIOpenSocket(&s, argv[optind], tx_id, rx_id, use_daemon)) < 0) {
		printf("Error connecting to vehicle: %s\n", strerror(errno));
		exit(EXIT_FAILURE);
	}

	printf("Supported commands:\n");

	OBDIICommandSet supportedCommands = OBDIIGetSupportedCommands(&s);

	for (i = 0; i < supportedCommands.numCommands; ++i) {
		OBDIICommand* command = supportedCommands.commands[i];
		printf("%i: mode %02x, PID %02x: %s\n", i, OBDIICommandGetMode(command), OBDIICommandGetPID(command),
			   command->name);
	}

	while (1) {
		// Print prompt
		printf("> ");

		// Get user's selection
		char line[20];
		if (!fgets(line, sizeof(line), stdin)) {
			break;
		}

		int selection;
		char option[3];
		char optionArg[11];
		int numScanned;

		if ((numScanned = sscanf(line, "%d %2s %10s", &selection, option, optionArg)) > 0) {
			int repeatQuery = numScanned == 2 && strcmp(option, "-p") == 0;
			int repeatInterval = (repeatQuery && numScanned == 3) ? atoi(optionArg) : 1000;	 // milliseconds

			if (selection >= 0 && selection < supportedCommands.numCommands) {
				OBDIICommand* command = supportedCommands.commands[selection];

				printf("Querying mode %02x PID %02x...\n", OBDIICommandGetMode(command), OBDIICommandGetPID(command));

				do {
					OBDIIResponse response = OBDIIPerformQuery(&s, command);

					if (response.success) {
						printf("Retrieved: ");

						if (command->responseType == OBDIIResponseTypeNumeric) {
							printf("%.2f", response.numericValue);
						} else if (command->responseType == OBDIIResponseTypeBitfield) {
							printf("%08x", response.bitfieldValue);
						} else if (command->responseType == OBDIIResponseTypeString) {
							printf("%s", response.stringValue);
						} else if (command->responseType == OBDIIResponseTypeOther) {
							printf("Unimplemented!");
						}

						printf("\n");
					} else {
						printf("Error retrieving data. Please try again!\n");
					}

					OBDIIResponseFree(&response);

					if (repeatQuery) {
						struct timespec delay;
						delay.tv_sec = repeatInterval / 1000;
						delay.tv_nsec = (repeatInterval % 1000) * 1000000;

						nanosleep(&delay, NULL);

						if (interrupted) {
							// We received SIGINT signal, so break out of loop
							interrupted = 0;
							break;
						}
					}
				} while (repeatQuery);
			} else {
				printf("%d is not a valid command!\n", selection);
			}
		} else {
			printf("Invalid input!\n");
		}
	}

	OBDIICommandSetFree(&supportedCommands);

	OBDIICloseSocket(&s);

	return 0;
}
