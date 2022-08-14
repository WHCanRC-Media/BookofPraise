
set SCRIPT_DIR=%~dp0
cd %SCRIPT_DIR%

py -m venv venv

call venv\bin\activate.bat
py -m pip install --upgrade pip
py -m pip install opencv-python
py -m pip install flask

set HYMN_USAGE_TXT=~/test.txt
py index.py
