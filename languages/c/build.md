# C Build System Patterns

Pure build system reference for C projects. These patterns apply to any domain.

---

## CMake

### Project Root CMakeLists.txt

```cmake
cmake_minimum_required(VERSION 3.16)
project(<project_name> C)

set(CMAKE_C_STANDARD <standard>)       # e.g. 11, 17, 23
set(CMAKE_C_STANDARD_REQUIRED ON)

# Include paths
include_directories(src/include)
```

### Per-Module Library

One `add_library` per module, with dependencies linked via `target_link_libraries`:

```cmake
# Module: <module_name>
add_library(<module_name>
    src/<module_name>/<module_name>.c
)
target_link_libraries(<module_name> PRIVATE <dependency_1> <dependency_2>)
```

Dependencies come from the module spec's `depends_on` list. Each entry maps to another `add_library` target.

### Per-Module Test Executable

```cmake
# Tests for <module_name>
add_executable(test_<module_name> tests/<module_name>_test.c)
target_link_libraries(test_<module_name> PRIVATE <module_name>)
```

### Enabling CTest

```cmake
enable_testing()
add_test(NAME test_<module_name> COMMAND test_<module_name>)
```

### External Package Integration

External packages use the `build_integration.cmake` string from the architecture spec:

```cmake
# External: <package_name>
find_package(<package_name> REQUIRED)
target_link_libraries(<module_name> PRIVATE <package_name>::<component>)
```

Or for SDK-provided packages:

```cmake
# SDK integration — follows build_integration.cmake from architecture spec
<build_integration_cmake_string>
```

### Complete Example

```cmake
cmake_minimum_required(VERSION 3.16)
project(my_project C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

include_directories(src/include)

# External packages
find_package(SomeLib REQUIRED)

# Module: utils
add_library(utils
    src/utils/utils.c
)

# Module: core
add_library(core
    src/core/core.c
)
target_link_libraries(core PRIVATE utils SomeLib::SomeLib)

# Module: app
add_library(app
    src/app/app.c
)
target_link_libraries(app PRIVATE core utils)

# Tests
enable_testing()

add_executable(test_utils tests/utils_test.c)
target_link_libraries(test_utils PRIVATE utils)
add_test(NAME test_utils COMMAND test_utils)

add_executable(test_core tests/core_test.c)
target_link_libraries(test_core PRIVATE core)
add_test(NAME test_core COMMAND test_core)

add_executable(test_app tests/app_test.c)
target_link_libraries(test_app PRIVATE app)
add_test(NAME test_app COMMAND test_app)
```

---

## Make

### Makefile Structure

```makefile
# Compiler and flags
CC = gcc
CFLAGS = -Wall -Wextra -Werror -std=c<standard>
INCLUDES = -Isrc/include

# Source and object files per module
<MODULE_A>_SRC = src/<module_a>/<module_a>.c
<MODULE_A>_OBJ = $(<MODULE_A>_SRC:.c=.o)

<MODULE_B>_SRC = src/<module_b>/<module_b>.c
<MODULE_B>_OBJ = $(<MODULE_B>_SRC:.c=.o)

# All objects
ALL_OBJ = $(<MODULE_A>_OBJ) $(<MODULE_B>_OBJ)

# Phony targets
.PHONY: all clean test

all: $(ALL_OBJ)

clean:
	rm -f $(ALL_OBJ) test_<module_a> test_<module_b>

# Pattern rule for .c -> .o
%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

# Test targets
test_<module_a>: tests/<module_a>_test.c $(<MODULE_A>_OBJ)
	$(CC) $(CFLAGS) $(INCLUDES) $^ -o $@

test_<module_b>: tests/<module_b>_test.c $(<MODULE_B>_OBJ) $(<MODULE_A>_OBJ)
	$(CC) $(CFLAGS) $(INCLUDES) $^ -o $@

test: test_<module_a> test_<module_b>
	./test_<module_a>
	./test_<module_b>
```

### External Package Integration (Make)

External packages append flags from `build_integration.make` in the architecture spec:

```makefile
# External: <package_name>
CFLAGS += -I/path/to/external/include
LDFLAGS += -L/path/to/external/lib -l<library>
```

Or using pkg-config for system packages:

```makefile
CFLAGS += $(shell pkg-config --cflags <package>)
LDFLAGS += $(shell pkg-config --libs <package>)
```

### Dependency Tracking

For automatic header dependency tracking:

```makefile
DEPFLAGS = -MMD -MP
CFLAGS += $(DEPFLAGS)

-include $(ALL_OBJ:.o=.d)
```

---

## Meson

### Project Root meson.build

```meson
project('<project_name>', 'c',
  version: '0.1.0',
  default_options: ['c_std=c<standard>'],
)

inc = include_directories('src/include')
```

### Per-Module Library

```meson
<module_name>_lib = static_library('<module_name>',
    'src/<module_name>/<module_name>.c',
    include_directories: inc,
)
```

With dependencies on other modules:

```meson
<module_name>_lib = static_library('<module_name>',
    'src/<module_name>/<module_name>.c',
    include_directories: inc,
    link_with: [<dependency_a>_lib, <dependency_b>_lib],
)
```

### Per-Module Test

```meson
test('test_<module_name>',
    executable('test_<module_name>',
        'tests/<module_name>_test.c',
        include_directories: inc,
        link_with: <module_name>_lib,
    ),
)
```

### External Package Integration (Meson)

External packages use `dependency()` calls from `build_integration.meson` in the architecture spec:

```meson
<package_name>_dep = dependency('<package_name>')

<module_name>_lib = static_library('<module_name>',
    'src/<module_name>/<module_name>.c',
    include_directories: inc,
    dependencies: [<package_name>_dep],
)
```

### Complete Example

```meson
project('my_project', 'c',
  version: '0.1.0',
  default_options: ['c_std=c11'],
)

inc = include_directories('src/include')

# External packages
some_dep = dependency('somelib')

# Module: utils
utils_lib = static_library('utils',
    'src/utils/utils.c',
    include_directories: inc,
)

# Module: core (depends on utils + external)
core_lib = static_library('core',
    'src/core/core.c',
    include_directories: inc,
    link_with: [utils_lib],
    dependencies: [some_dep],
)

# Module: app (depends on core, utils)
app_lib = static_library('app',
    'src/app/app.c',
    include_directories: inc,
    link_with: [core_lib, utils_lib],
)

# Tests
test('test_utils',
    executable('test_utils',
        'tests/utils_test.c',
        include_directories: inc,
        link_with: utils_lib,
    ),
)

test('test_core',
    executable('test_core',
        'tests/core_test.c',
        include_directories: inc,
        link_with: core_lib,
    ),
)

test('test_app',
    executable('test_app',
        'tests/app_test.c',
        include_directories: inc,
        link_with: app_lib,
    ),
)
```

---

## Build System Selection

The `build_system` field in the architecture spec determines which build files to generate:

| Value | Files Generated |
|-------|----------------|
| `cmake` | `CMakeLists.txt` |
| `make` | `Makefile` |
| `meson` | `meson.build` |

Only one build system is generated per project. The choice is specified in the architecture spec.
