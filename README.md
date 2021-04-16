

If you're targeting a specific python version, you can set `PYTHON_SYS_EXECUTABLE` to the python executable in your machine with that version. 

e.g., on Windows via Git Bash:
```sh
PYTHON_SYS_EXECUTABLE=/c/Users/nickj/dev/proj/leap/keras-hannd/env/Scripts/python cargo build --release
```

# Ubuntu

Make sure you have libssl and libpython installed:
```sh
sudo apt-get install libssl-dev
sudo apt-get install libpython3-dev
```

If you encounter errors building `pyo3`, you may need to check whether it can find yor python and any related python dev dependencies: https://pyo3.rs/v0.10.1/building_and_distribution.html
