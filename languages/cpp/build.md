# C++ Build System Patterns

Pure build system reference for C++ projects. These patterns apply to any domain.

---

## CMake

### Project Root CMakeLists.txt

```cmake
cmake_minimum_required(VERSION 3.16)
project(<project_name> CXX)

set(CMAKE_CXX_STANDARD <standard>)     # e.g. 17, 20, 23
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Include paths
include_directories(src/include)
```

### Per-Module Library

One `add_library` per module, with dependencies linked via `target_link_libraries`:

```cmake
# Module: <module_name>
add_library(<module_name>
    src/<module_name>/<module_name>.cpp
)
target_link_libraries(<module_name> PRIVATE <dependency_1> <dependency_2>)
```

Dependencies come from the module spec's `depends_on` list. Each entry maps to another `add_library` target.

### Per-Module Test Executable

```cmake
# Tests for <module_name>
add_executable(test_<module_name> tests/<module_name>_test.cpp)
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

### Complete Example

```cmake
cmake_minimum_required(VERSION 3.16)
project(my_project CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include_directories(src/include)

# External packages
find_package(SomeLib REQUIRED)

# Module: utils
add_library(utils
    src/utils/utils.cpp
)

# Module: core
add_library(core
    src/core/core.cpp
)
target_link_libraries(core PRIVATE utils SomeLib::SomeLib)

# Module: app
add_library(app
    src/app/app.cpp
)
target_link_libraries(app PRIVATE core utils)

# Tests
enable_testing()

add_executable(test_utils tests/utils_test.cpp)
target_link_libraries(test_utils PRIVATE utils)
add_test(NAME test_utils COMMAND test_utils)

add_executable(test_core tests/core_test.cpp)
target_link_libraries(test_core PRIVATE core)
add_test(NAME test_core COMMAND test_core)

add_executable(test_app tests/app_test.cpp)
target_link_libraries(test_app PRIVATE app)
add_test(NAME test_app COMMAND test_app)
```

---

## Make

### Makefile Structure

```makefile
# Compiler and flags
CXX = g++
CXXFLAGS = -Wall -Wextra -Werror -std=c++<standard>
INCLUDES = -Isrc/include

# Source and object files per module
<MODULE_A>_SRC = src/<module_a>/<module_a>.cpp
<MODULE_A>_OBJ = $(<MODULE_A>_SRC:.cpp=.o)

<MODULE_B>_SRC = src/<module_b>/<module_b>.cpp
<MODULE_B>_OBJ = $(<MODULE_B>_SRC:.cpp=.o)

# All objects
ALL_OBJ = $(<MODULE_A>_OBJ) $(<MODULE_B>_OBJ)

# Phony targets
.PHONY: all clean test

all: $(ALL_OBJ)

clean:
	rm -f $(ALL_OBJ) test_<module_a> test_<module_b>

# Pattern rule for .cpp -> .o
%.o: %.cpp
	$(CXX) $(CXXFLAGS) $(INCLUDES) -c $< -o $@

# Test targets
test_<module_a>: tests/<module_a>_test.cpp $(<MODULE_A>_OBJ)
	$(CXX) $(CXXFLAGS) $(INCLUDES) $^ -o $@

test_<module_b>: tests/<module_b>_test.cpp $(<MODULE_B>_OBJ) $(<MODULE_A>_OBJ)
	$(CXX) $(CXXFLAGS) $(INCLUDES) $^ -o $@

test: test_<module_a> test_<module_b>
	./test_<module_a>
	./test_<module_b>
```

### External Package Integration (Make)

External packages append flags from `build_integration.make` in the architecture spec:

```makefile
# External: <package_name>
CXXFLAGS += -I/path/to/external/include
LDFLAGS += -L/path/to/external/lib -l<library>
```

Or using pkg-config for system packages:

```makefile
CXXFLAGS += $(shell pkg-config --cflags <package>)
LDFLAGS += $(shell pkg-config --libs <package>)
```

### Dependency Tracking

For automatic header dependency tracking:

```makefile
DEPFLAGS = -MMD -MP
CXXFLAGS += $(DEPFLAGS)

-include $(ALL_OBJ:.o=.d)
```

---

## Meson

### Project Root meson.build

```meson
project('<project_name>', 'cpp',
  version: '0.1.0',
  default_options: ['cpp_std=c++<standard>'],
)

inc = include_directories('src/include')
```

Note the project type is `'cpp'` (not `'c'`), and the standard option is `cpp_std` (not `c_std`).

### Per-Module Library

```meson
<module_name>_lib = static_library('<module_name>',
    'src/<module_name>/<module_name>.cpp',
    include_directories: inc,
)
```

With dependencies on other modules:

```meson
<module_name>_lib = static_library('<module_name>',
    'src/<module_name>/<module_name>.cpp',
    include_directories: inc,
    link_with: [<dependency_a>_lib, <dependency_b>_lib],
)
```

### Per-Module Test

```meson
test('test_<module_name>',
    executable('test_<module_name>',
        'tests/<module_name>_test.cpp',
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
    'src/<module_name>/<module_name>.cpp',
    include_directories: inc,
    dependencies: [<package_name>_dep],
)
```

### Complete Example

```meson
project('my_project', 'cpp',
  version: '0.1.0',
  default_options: ['cpp_std=c++17'],
)

inc = include_directories('src/include')

# External packages
some_dep = dependency('somelib')

# Module: utils
utils_lib = static_library('utils',
    'src/utils/utils.cpp',
    include_directories: inc,
)

# Module: core (depends on utils + external)
core_lib = static_library('core',
    'src/core/core.cpp',
    include_directories: inc,
    link_with: [utils_lib],
    dependencies: [some_dep],
)

# Module: app (depends on core, utils)
app_lib = static_library('app',
    'src/app/app.cpp',
    include_directories: inc,
    link_with: [core_lib, utils_lib],
)

# Tests
test('test_utils',
    executable('test_utils',
        'tests/utils_test.cpp',
        include_directories: inc,
        link_with: utils_lib,
    ),
)

test('test_core',
    executable('test_core',
        'tests/core_test.cpp',
        include_directories: inc,
        link_with: core_lib,
    ),
)

test('test_app',
    executable('test_app',
        'tests/app_test.cpp',
        include_directories: inc,
        link_with: app_lib,
    ),
)
```

---

## C++ Standard Selection

Common standard values and their Meson/CMake names:

| Standard | CMake `CMAKE_CXX_STANDARD` | Meson `cpp_std` |
|----------|---------------------------|-----------------|
| C++11 | `11` | `c++11` |
| C++14 | `14` | `c++14` |
| C++17 | `17` | `c++17` |
| C++20 | `20` | `c++20` |
| C++23 | `23` | `c++23` |

### Feature Availability by Standard

| Feature | Minimum Standard |
|---------|-----------------|
| `enum class` | C++11 |
| `auto`, range-for | C++11 |
| `std::make_unique` | C++14 |
| `std::optional` | C++17 |
| `std::string_view` | C++17 |
| `if constexpr` | C++17 |
| `std::span` | C++20 |
| `concepts` | C++20 |
| `std::expected` | C++23 |

Use features that are available at or below the project's configured standard. If `std::expected` is needed but the standard is below C++23, use the error enum return pattern or a project-level `Result<T, E>` type instead.

---

## Build System Selection

The `build_system` field in the architecture spec determines which build files to generate:

| Value | Files Generated |
|-------|----------------|
| `cmake` | `CMakeLists.txt` |
| `make` | `Makefile` |
| `meson` | `meson.build` |

Only one build system is generated per project. The choice is specified in the architecture spec.
