#include "Token.hpp"

namespace Kod
{
    Location::Location(const std::wstring& file_path, uint32_t line, uint32_t column) :
        m_file_path(file_path),
        m_line(line),
        m_column(column)
    {}

    std::wostream& operator<<(std::wostream& os, const Location& location)
    {
        os << location.m_file_path.c_str() << ":" << location.m_line << ":" << location.m_column;
        return os;
    }

    Token::Token(const Location location, const TokenType type, const std::wstring& value) :
        m_location(location),
        m_type(type),
        m_value(value)
    {}

    std::wostream& operator<<(std::wostream& os, const Token& token)
    {
        os << token.m_location << ": (" << static_cast<uint32_t>(token.m_type) << "): " << token.m_value;
        return os;
    }
}
