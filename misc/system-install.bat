@echo off
net session >nul 2>&1 || (
    echo This script requires administrator privileges.
    pause
    exit /b
)

set "DEST=C:\Program Files\bRAC"
mkdir "%DEST%" 2>nul
xcopy "." "%DEST%\" /E /I /H /Y >nul

for /d %%u in ("C:\Users\*") do (
    if exist "%%u\AppData\Roaming\Microsoft\Windows\Desktop" (
        call :s "%%u\AppData\Roaming\Microsoft\Windows\Desktop\bRAC.lnk" "%DEST%\bRAC.exe"
    ) else if exist "%%u\Desktop" (
        call :s "%%u\Desktop\bRAC.lnk" "%DEST%\bRAC.exe"
    )
)
exit /b

:s
set "v=%TEMP%\_s.vbs"
> "%v%" echo Set o=CreateObject("WScript.Shell")
>>"%v%" echo Set l=o.CreateShortcut("%~1")
>>"%v%" echo l.TargetPath="%~2"
>>"%v%" echo l.WorkingDirectory="%~dp2"
>>"%v%" echo l.Save
wscript "%v%" >nul
del "%v%" >nul
exit /b
