@echo off
chcp 65001 >nul
cd /d "%~dp0"
if exist "output\观棋\guanqi.exe" (
  start "" "output\观棋\guanqi.exe"
  exit /b
)
where node >nul 2>nul
if errorlevel 1 (
  echo 请先安装 Node.js 24，或使用 output 目录下的桌面程序。
  pause
  exit /b 1
)
if not exist "dist\index.html" (
  call npm install
  if errorlevel 1 goto failed
  call npm run build
  if errorlevel 1 goto failed
)
start "" "http://127.0.0.1:1422"
node scripts\serve.mjs
pause
exit /b
:failed
echo 启动失败，请查看上方提示。
pause
