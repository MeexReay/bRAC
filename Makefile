.PHONY: clean install uninstall build_linux build_windows build_all

TARGETS = \
	i686-unknown-linux-gnu \
	i686-unknown-linux-musl \
	x86_64-unknown-linux-none \
	x86_64-unknown-linux-gnu \
	x86_64-unknown-linux-musl \
	aarch64-unknown-linux-gnu \
	aarch64-unknown-linux-musl

install: target/release/bRAC
	mkdir -p ~/.local
	mkdir -p ~/.local/bin
	mkdir -p ~/.local/share
	cp $< ~/.local/bin/bRAC
	chmod +x ~/.local/bin/bRAC
	mkdir ~/.local/share/bRAC -p
	cp misc/bRAC.png ~/.local/share/bRAC/icon.png
	./misc/create-desktop.sh > ~/.local/share/applications/ru.themixray.bRAC.desktop
uninstall:
	rm -rf ~/.config/bRAC ~/.local/share/bRAC
	rm -f ~/.local/share/applications/ru.themixray.bRAC.desktop
target/release/bRAC:
	cargo build -r

build_all: build_linux build_windows

build_linux:
	mkdir -p build
	mkdir -p build/linux
	for target in $(TARGETS); do \
		cargo build -r --target $$target; \
		cp target/$$target/bRAC build/linux/$$target-bRAC; \
	done

build_windows:
	echo "Windows build is in development!!!"

clean: 
	cargo clean
	rm -rf build
