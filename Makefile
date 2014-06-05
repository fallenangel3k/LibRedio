CC=rustc

CFLAGS=-L ./lib -A unused-variable -A unused-imports -A non_snake_case_functions
OBJ = ./lib/libkissfft*.rlib ./lib/librtlsdr*.rlib ./lib/libdsputils*.rlib ./lib/libkpn*.rlib ./lib/libbitfount*.rlib ./lib/liblibusb*.rlib ./lib/libusb*.rlib
# ./lib/libtoml*.rlib ./lib/liboutlet*.rlib
ifeq ($(ARCH),arm)
CFLAGS+= --target arm-unknown-linux-gnueabihf -C linker=arm-linux-gnueabihf-gcc -C link-args=-Wl,-rpath-link,$(PWD)/lib/ -C target-feature=+vfp3,+v7,+neon,+vfp4
else
OBJ += ./lib/libsdl2*.rlib ./lib/libvidsink2*.rlib ./lib/libpasimple*.rlib
endif

all: $(OBJ)
	rm -rf ratpak/stage3.rs
	make -C ./ratpak
	$(CC) $(CFLAGS) -O -o final ./ratpak/stage3.rs

test:
	$(CC) $(CFLAGS) ./src/test.rs

./lib/libsndfile*.rlib:
	mkdir -p lib
	$(CC) $(CFLAGS) --crate-type=lib ./src/sndfile.rs
	mv -f *rlib lib

./lib/libsdl2*.rlib:
	mkdir -p lib
	make -C ../rust-sdl2
	mv ../rust-sdl2/build/lib/libsdl2* ./lib

./lib/libsdl*.rlib:
	mkdir -p lib
	$(CC) $(CFLAGS) ../rust-sdl/src/sdl/lib.rs
	mv *rlib lib


./lib/lib%.rlib: ./src/%.rs
	mkdir -p lib
	$(CC) $(CFLAGS) --crate-type=lib $<
	mv -f *rlib lib

clean:
	rm -fr lib/*rlib bin
