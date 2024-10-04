ifeq ($(OS),Windows_NT)
    NAME := $(EXE).exe
	BIN := crinnge.exe
else
    NAME := $(EXE)
	BIN := crinnge
endif

rule:
	RUSTFLAGS="-Ctarget-cpu=native" cargo build --release --bin crinnge
	cp ./target/release/$(BIN) $(NAME)
