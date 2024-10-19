#pragma once

#include "Token.hpp"

namespace Kod
{
    class Lexer
    {
    public:
        Lexer(const std::wstring& content, const std::wstring& file_path);
        virtual ~Lexer() = default;
        Lexer(const Lexer&) = delete;
        Lexer& operator=(const Lexer&) = delete;
        Lexer(Lexer&&) = delete;
        Lexer& operator=(Lexer&&) = delete;

        Token get_next_token();
        Token peek_token();

    private:
        void _skip_whitespace();
        void _skip_comments();
        void _skip_until(const wchar_t character);
        wchar_t _get_current_char() const;
        wchar_t _peek_char() const;
        void _advance();
        inline bool _can_advance() const;
        bool _is_start_of_comments() const;
        Token _collect_string();
        Token _collect_number();
        Token _collect_identifier();
        Token _collect_symbol();

        static bool _s_is_bin(const wchar_t character) { return std::isdigit(character) && (L'1' >= character); }
        static bool _s_is_oct(const wchar_t character) { return std::isdigit(character) && (L'8' >= character); }

    private:
        const std::wstring m_content;
        Location m_location;
        uint32_t m_index;
    };
}
