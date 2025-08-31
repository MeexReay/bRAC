.PHONY: clean install uninstall package

target/release/bRAC:
	cargo build -r
	
install: target/release/bRAC
	mkdir -p /usr/bin
	mkdir -p /usr/share
	cp $< /usr/bin/bRAC
	chmod +x /usr/bin/bRAC
	cp misc/bRAC.png /usr/share/pixmaps/ru.themixray.bRAC.png
	chmod +x misc/create-desktop.sh
	./misc/create-desktop.sh > /usr/share/applications/ru.themixray.bRAC.desktop
uninstall:
	rm -f /usr/bin/bRAC
	rm -f /usr/share/applications/ru.themixray.bRAC.desktop
	rm -f /usr/share/pixmaps/ru.themixray.bRAC.png

package:
	./misc/build.sh
	mkdir -p package
	for i in $$( ls build/*.zip ); do \
		mv $$i package/bRAC-$$(basename $$i); \
	done
	
clean: 
	rm -rf build package target
