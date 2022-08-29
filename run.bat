
set SCRIPT_DIR=%~dp0
cd %SCRIPT_DIR%

py -m venv venv

call venv\bin\activate.bat
py -m pip install --quiet --upgrade pip opencv-python flask

REM set HYMN_USAGE_TXT=~/test.txt
py index.py
