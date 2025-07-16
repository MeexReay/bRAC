.PHONY: clean install uninstall package

target/release/bRAC:
	cargo build -r
	
install: target/release/bRAC
	mkdir -p ~/.local
	mkdir -p ~/.local/bin
	mkdir -p ~/.local/share
	cp $< ~/.local/bin/bRAC
	chmod +x ~/.local/bin/bRAC
	mkdir ~/.local/share/bRAC -p
	cp misc/bRAC.png ~/.local/share/bRAC/icon.png
	chmod +x misc/create-desktop.sh
	./misc/create-desktop.sh > ~/.local/share/applications/ru.themixray.bRAC.desktop
uninstall:
	rm -rf ~/.config/bRAC ~/.local/share/bRAC
	rm -f ~/.local/bin/bRAC ~/.local/share/applications/ru.themixray.bRAC.desktop

package:
	./misc/build.sh
	mkdir -p package
	for i in $$( ls build/*.zip ); do \
		mv $$i package/bRAC-$$(basename $$i); \
	done
	
clean: 
	rm -rf build package target
