build-windows:
	mkdir -p build
	mkdir -p build/windows-x86_64
	# cargo build -r --target x86_64-pc-windows-gnu
	# cp target/x86_64-pc-windows-gnu/release/bRAC
	curl -s https://api.github.com/repos/wingtk/gvsbuild/releases/latest \
		| grep -o ".*browser_download_url.*GTK4_Gvsbuild.*_x64.zip.*" \
		| cut -d : -f 2,3 \
		| tr -d \" \
		| wget -O build/windows-x86_64/gtk4.zip -qi -
	unzip build/windows-x86_64/gtk4.zip -d build/windows-x86_64
	rm build/windows-x86_64/gtk4.zip
	echo 'Set oWS = WScript.CreateObject("WScript.Shell")' > build/windows-x86_64/gen_link.vbs
	echo 'Set oLink = oWS.CreateShortcut("build/windows-x86_64/bRAC.lnk")' >> build/windows-x86_64/gen_link.vbs
	echo 'oLink.TargetPath = "bin\\bRAC.exe"' >> build/windows-x86_64/gen_link.vbs
	echo 'oLink.Description = "Relative shortcut to bin\\myapp.exe"' >> build/windows-x86_64/gen_link.vbs
	echo 'oLink.Save' >> build/windows-x86_64/gen_link.vbs
	wine wscript.exe build/windows-x86_64/gen_link.vbs
	rm build/windows-x86_64/gen_link.vbs