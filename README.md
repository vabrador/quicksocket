

If you're targeting a specific python version, you can set `PYTHON_SYS_EXECUTABLE` to the python executable in your machine with that version. 

e.g., on Windows via Git Bash:
```sh
PYTHON_SYS_EXECUTABLE=/c/Users/nickj/dev/proj/leap/keras-hannd/env/Scripts/python cargo build --release
```
