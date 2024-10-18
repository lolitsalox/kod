#include "FileUtils.hpp"

namespace FileUtils
{
    std::wstring read_whole_file(const std::filesystem::path& file_path)
    {
        std::wifstream file(file_path);

        if (!file.is_open())
        {
            throw KodException(KodStatus::KODSTATUS_FILEUTILS_READ_WHOLE_FILE_COULDNT_OPEN_FILE);
        }

        file.seekg(0, file.end);
        std::streampos length = file.tellg();
        file.seekg(0, file.beg);

        std::wstring buffer(length, '\0');

        file.read(const_cast<wchar_t*>(buffer.c_str()), length);
        return buffer;
    }
}
