.PHONY: clean build

build: build/windows-x86_64 build/linux-x86_64

build/windows-x86_64:
	mkdir -p build
	mkdir -p $@
	cargo build -r -F winapi --target x86_64-pc-windows-gnu
	curl -s https://api.github.com/repos/wingtk/gvsbuild/releases/latest \
		| grep -o ".*browser_download_url.*GTK4_Gvsbuild.*_x64.zip.*" \
		| cut -d : -f 2,3 \
		| tr -d \" \
		| wget -O $@/gtk4.zip -qi -
	unzip $@/gtk4.zip -d $@
	rm $@/gtk4.zip
	mv $@/bin/* $@/
	cp target/x86_64-pc-windows-gnu/release/bRAC.exe $@
	rm -r $@/bin

build/linux-x86_64:
	mkdir -p build
	mkdir -p $@
	cargo build -r -F libnotify --target x86_64-unknown-linux-gnu
	# patchbin target/x86_64-unknown-linux-gnu/release/bRAC
	cp target/x86_64-unknown-linux-gnu/release/bRAC $@
	cp ru.themixray.bRAC.png $@
	cp ru.themixray.bRAC.desktop $@
	cp install.sh $@
	cp uninstall.sh $@

clean:
	rm -r build