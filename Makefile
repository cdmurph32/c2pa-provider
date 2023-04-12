# c2pa-provider Makefile

CAPABILITY_ID = "wasmcloud:adobe:c2pa_provider"
NAME = "c2pa-provider"
VENDOR = "Adobe"
PROJECT = c2pa_provider
VERSION = 0.1.0
REVISION = 0

include ./provider.mk

test::
	cargo clippy --all-targets --all-features

