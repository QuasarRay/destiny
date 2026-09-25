from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class NativeContractTests(unittest.TestCase):
    def test_c_header_is_valid_cxx17_and_raii_wrapper_is_self_contained(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            source = Path(temp_dir) / "smoke.cpp"
            source.write_text(
                "#include <type_traits>\n"
                '#include "destiny_bevy_compat.hpp"\n'
                '#include "destiny_carbon.hpp"\n'
                "int main() {\n"
                "  static_assert(DBC_ABI_VERSION == 1u);\n"
                "  dbc::Buffer empty;\n"
                "  static_assert(!std::is_copy_constructible<destiny::Ballpark>::value);\n"
                "  static_assert(std::is_copy_constructible<destiny::Ball>::value);\n"
                "  static_assert(std::is_same<decltype(std::declval<destiny::Ballpark&>().GetCenterDist(1, 2)), std::optional<double>>::value);\n"
                "  return empty.empty() ? 0 : 1;\n"
                "}\n",
                encoding="utf-8",
            )
            subprocess.run(
                [
                    "c++",
                    "-std=c++17",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-pedantic",
                    "-fsyntax-only",
                    "-I",
                    str(ROOT / "include"),
                    str(source),
                ],
                check=True,
            )

    def test_cpp_json_and_invalid_buffer_guards_execute(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            source = Path(temp_dir) / "guards.cpp"
            executable = Path(temp_dir) / "guards"
            source.write_text(
                '#include "destiny_carbon.hpp"\n'
                '#include <string>\n'
                'extern "C" void dbc_buffer_free(DbcBuffer) {}\n'
                'int main() {\n'
                '  using destiny::detail::json_string;\n'
                '  using destiny::detail::require_json_value;\n'
                '  using destiny::detail::result_from_response;\n'
                '  if (json_string("a\\\"\\n") != "\\\"a\\\\\\\"\\\\n\\\"") return 1;\n'
                '  if (result_from_response("{\\\"ok\\\":true,\\\"result\\\":42}") != "42") return 2;\n'
                '  try { (void)result_from_response("{\\\"ok\\\":true,\\\"result\\\":1,\\\"result\\\":2}"); return 3; } catch (const dbc::Error&) {}\n'
                '  try { require_json_value("null true", "value"); return 4; } catch (const dbc::Error&) {}\n'
                '  try { dbc::Buffer invalid(DbcBuffer{nullptr, 1, 0}); (void)invalid.string(); return 5; } catch (const dbc::Error&) {}\n'
                '  return 0;\n'
                '}\n',
                encoding="utf-8",
            )
            subprocess.run(
                [
                    "c++",
                    "-std=c++17",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-pedantic",
                    "-I",
                    str(ROOT / "include"),
                    str(source),
                    "-o",
                    str(executable),
                ],
                check=True,
            )
            subprocess.run([str(executable)], check=True)

    def test_c_abi_uses_opaque_runtime_and_owned_buffers(self) -> None:
        header = (ROOT / "include" / "destiny_bevy_compat.h").read_text(encoding="utf-8")
        self.assertIn("typedef struct DbcRuntime DbcRuntime;", header)
        self.assertIn("dbc_runtime_call", header)
        self.assertIn("dbc_buffer_free", header)
        self.assertIn("DBC_ABI_VERSION 1u", header)

    def test_carbon_cmake_entry_points_and_target_spellings_are_present(self) -> None:
        build = (ROOT / "CMakeLists.txt").read_text(encoding="utf-8")
        config = (ROOT / "cmake" / "carbon-destinyConfig.cmake.in").read_text(encoding="utf-8")
        direct = (ROOT / "cmake" / "carbon-destiny.cmake").read_text(encoding="utf-8")
        for spelling in (
            "destiny::bevy_compat",
            "carbon-destiny::destiny",
            "Destiny",
            "destiny",
        ):
            self.assertIn(spelling, build + config)
        self.assertIn('DESTINATION "share/carbon-destiny"', build)
        self.assertIn("carbon-destinyLegacyConfig.cmake", build)
        self.assertIn("rust-toolchain.toml", build)
        self.assertIn("tests/native/c_abi_smoke.c", build)
        self.assertIn("carbon-destinyConfig.cmake", direct)


if __name__ == "__main__":
    unittest.main()
