#This target is to ensure accidental execution of Makefile as a bash script will not execute commands like rm in unexpected directories and exit gracefully.
.prevent_execution:
	exit 0

CC = gcc -x c

#remove @ for no make command prints
DEBUG=@

LIBRARY_INCLUDE_DIRS = -I src
LIBRARY_SRC_FILES=src/OBDII.c src/OBDIICommunication.c

DAEMON_SRC_FILES = src/OBDIIDaemon.c
DAEMON_INCLUDE_DIRS = -I src

CLI_DIR = src
CLI_INCLUDE_DIRS += -I $(CLI_DIR)
CLI_SRC_FILES=$(LIBRARY_SRC_FILES)

BUILD_DIR = build

CLI_TARGET_NAME = cli

COMPILER_FLAGS += -g

.PHONY:

all: cli shared daemon

cli:
	@mkdir -p $(BUILD_DIR)
	$(DEBUG) $(CC) $(CLI_DIR)/$(CLI_TARGET_NAME).c $(CLI_SRC_FILES) $(COMPILER_FLAGS) -o $(BUILD_DIR)/$(CLI_TARGET_NAME) $(CLI_INCLUDE_DIRS)

shared:
	@mkdir -p $(BUILD_DIR)
	$(DEBUG) $(CC) $(LIBRARY_SRC_FILES) $(COMPILER_FLAGS) -fpic -shared -o $(BUILD_DIR)/libobdii.so $(LIBRARY_INCLUDE_DIRS)

daemon:
	@mkdir -p $(BUILD_DIR)
	$(DEBUG)$(CC) $(COMPILER_FLAGS) $(DAEMON_SRC_FILES) $(DAEMON_INCLUDE_DIRS) -o $(BUILD_DIR)/obdiid

clean:
	rm -f $(BUILD_DIR)/*
