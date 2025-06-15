.PHONY: clean install uninstall

install: target/release/bRAC
	cp $< ~/.local/bin/bRAC
	chmod +x ~/.local/bin/bRAC
	mkdir ~/.local/share/bRAC -p
	cp misc/bRAC.png ~/.local/share/bRAC/icon.png
	cp misc/bRAC.desktop ~/.local/share/applications/ru.themixray.bRAC.desktop
uninstall:
	rm -rf ~/.config/bRAC ~/.local/share/bRAC
	rm -f ~/.local/share/applications/ru.themixray.bRAC.desktop
target/release/bRAC:
	cargo build -r
clean: 
	cargo clean