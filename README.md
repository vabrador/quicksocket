

You might need to activate your virtualenv to build targeting the correct version of Python:
```
source ~/dev/proj/leap/keras-hannd/env/Scripts/activate
```

Or (and?) you need to at least set the proper sys executable flag:
```
PYTHON_SYS_EXECUTABLE=/c/Users/nickj/dev/proj/leap/keras-hannd/env/Scripts/python cargo build --release
```
