.PHONY: clean build

build: build/windows-x86_64 build/linux-x86_64

build/windows-x86_64:
	mkdir -p build
	mkdir -p $@
	cargo build -r --target x86_64-pc-windows-gnu
	cp target/x86_64-pc-windows-gnu/release/bRAC $@/bin
	curl -s https://api.github.com/repos/wingtk/gvsbuild/releases/latest \
		| grep -o ".*browser_download_url.*GTK4_Gvsbuild.*_x64.zip.*" \
		| cut -d : -f 2,3 \
		| tr -d \" \
		| wget -O $@/gtk4.zip -qi -
	unzip $@/gtk4.zip -d $@
	rm $@/gtk4.zip
	mv $@/bin/* b$@/
	rm $@/bin

build/linux-x86_64:
	mkdir -p build
	mkdir -p $@
	cargo build -r --target x86_64-unknown-linux-gnu
	cp target/x86_64-pc-windows-gnu/release/bRAC $@

clean:
	rm -r build