@echo off
net session >nul 2>&1 || (
    echo This script requires administrator privileges.
    pause
    exit /b
)

set "TARGET=C:\Program Files\bRAC\bRAC.exe"

for /d %%u in ("C:\Users\*") do (
    call :d "%%u\AppData\Roaming\Microsoft\Windows\Desktop"
    call :d "%%u\Desktop"
)

cd /d "%TEMP%"
rmdir /s /q "C:\Program Files\bRAC"
exit /b

:d
if not exist "%~1" exit /b
for %%f in ("%~1\*.lnk") do (
    call :c "%%~f"
)
exit /b

:c
set "v=%TEMP%\_c.vbs"
> "%v%" echo Set o=CreateObject("WScript.Shell")
>>"%v%" echo Set l=o.CreateShortcut("%~1")
>>"%v%" echo WScript.Echo l.TargetPath
for /f "usebackq delims=" %%t in (`wscript //nologo "%v%"`) do (
    if /I "%%t"=="%TARGET%" del /f /q "%~1"
)
del "%v%" >nul
exit /b
