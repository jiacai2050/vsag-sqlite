

dev:
	VSAG_LIB_PATH=$(HOME)/vsag/build/src cargo +nightly build --no-default-features

sqlite:
	LD_LIBRARY_PATH=$(HOME)/vsag/build/src sqlite3 /tmp/db < vsag.sql
