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

    Token::Token(const Location location, const TokenType type, const std::wstring& value, const KeywordType keyword_type) :
        m_location(location),
        m_token_type(type),
        m_value(value),
        m_keyword_type(keyword_type),
        m_int_value(0),
        m_float_value(0.0f)
    {}

    TokenType Token::s_symbol_to_type(const std::wstring& symbol)
    {
        static const std::unordered_map<std::wstring, TokenType> symbol_to_type = {
            { L"::", TokenType::NAMESPACE },
            { L"&&", TokenType::BOOL_AND },
            { L"||", TokenType::BOOL_OR },
            { L"**", TokenType::POW },
            { L"//", TokenType::LINE_COMMENT },
            { L"/*", TokenType::MULTILINE_COMMENT_START },
            { L"*/", TokenType::MULTILINE_COMMENT_END },
            { L"->", TokenType::POINTER },
            { L"!=", TokenType::BOOL_NE },
            { L"==", TokenType::BOOL_EQ },
            { L"=>", TokenType::ARROW },
            { L"<<", TokenType::SHL },
            { L">>", TokenType::SHR },
            { L"<=", TokenType::BOOL_LTE },
            { L">=", TokenType::BOOL_GTE },
            { L"(", TokenType::LPAREN },
            { L")", TokenType::RPAREN },
            { L"[", TokenType::LBRACKET },
            { L"]", TokenType::RBRACKET },
            { L"{", TokenType::LBRACE },
            { L"}", TokenType::RBRACE },
            { L"=", TokenType::EQUALS },
            { L",", TokenType::COMMA },
            { L":", TokenType::COLON },
            { L";", TokenType::SEMI },
            { L"?", TokenType::QUESTION },
            { L"%", TokenType::MOD },
            { L"\\", TokenType::BACKSLASH },
            { L"#", TokenType::HASH },
            { L"@", TokenType::AT },
            { L"+", TokenType::PLUS },
            { L"-", TokenType::MINUS },
            { L"/", TokenType::DIV },
            { L"*", TokenType::MUL },
            { L"&", TokenType::AND },
            { L"|", TokenType::OR },
            { L"^", TokenType::HAT },
            { L"<", TokenType::BOOL_LT },
            { L">", TokenType::BOOL_GT },
            { L"~", TokenType::NOT },
            { L"!", TokenType::BOOL_NOT },
            { L".", TokenType::DOT },
            { L"+=", TokenType::PLUS_EQ },
            { L"-=", TokenType::MINUS_EQ },
            { L"/=", TokenType::DIV_EQ },
            { L"*=", TokenType::MUL_EQ },
            { L"%=", TokenType::MOD_EQ },
        };
        const auto iter = symbol_to_type.find(symbol);
        return symbol_to_type.end() == iter ? TokenType::UNKNOWN : (*iter).second;
    }

    KeywordType Token::s_keyword_to_type(const std::wstring& keyword)
    {
        static const std::unordered_map<std::wstring, KeywordType> keyword_to_type = {
            { L"null", KeywordType::NULL_K },
            { L"fn", KeywordType::FN },
            { L"if", KeywordType::IF },
            { L"else", KeywordType::ELSE },
            { L"while", KeywordType::WHILE },
            { L"for", KeywordType::FOR },
            { L"return", KeywordType::RETURN },
            { L"import", KeywordType::IMPORT },
            { L"as", KeywordType::AS },
            { L"from", KeywordType::FROM },
            { L"break", KeywordType::BREAK },
            { L"continue", KeywordType::CONTINUE },
            { L"true", KeywordType::TRUE },
            { L"false", KeywordType::FALSE },
        };
        const auto iter = keyword_to_type.find(keyword);
        return keyword_to_type.end() == iter ? KeywordType::UNKNOWN : (*iter).second;
    }

    bool Token::_s_is_symbol(wchar_t character)
    {
        static const std::wstring symbols(L"()[]{}=@#,.:;?\\+-/*%&|^<>!~");
        return std::wstring::npos != symbols.find(character);
    }

    std::wostream& operator<<(std::wostream& os, const Token& token)
    {
        os << token.m_location << ": (" << static_cast<uint32_t>(token.m_token_type) << "): " << token.m_value;
        return os;
    }
}
