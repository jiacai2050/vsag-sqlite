

dev:
	VSAG_LIB_PATH=$(HOME)/vsag/build/src cargo build --no-default-features

sqlite:
	LD_LIBRARY_PATH=$(HOME)/vsag/build/src sqlite3 /tmp/vsag-sqlite.db < test.sql
