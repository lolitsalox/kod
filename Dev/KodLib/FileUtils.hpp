#pragma once

#include "BufferUtils.hpp"

namespace FileUtils
{
    [[nodiscard]] std::wstring read_whole_file(const std::filesystem::path& file_path);
}
