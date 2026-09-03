@echo off
set "DOTNET_ROOT=%LOCALAPPDATA%\Microsoft\dotnet"
set "PATH=%LOCALAPPDATA%\Microsoft\dotnet;%PATH%"
start "" "%~dp0app\Sweepie.App\bin\x64\Release\net8.0-windows10.0.19041.0\win-x64\Sweepie.App.exe"
