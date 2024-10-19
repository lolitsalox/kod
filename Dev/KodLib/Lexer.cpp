#include "BufferUtils.hpp"
#include "Lexer.hpp"

namespace Kod
{
    Lexer::Lexer(const std::wstring& content, const std::wstring& file_path) :
        m_content(content),
        m_location(file_path, 1, 1),
        m_index(0)
    {}

    Token Lexer::get_next_token()
    {
        for (;;)
        {
            if (!_can_advance())
            {
                return Token(m_location, TokenType::END_OF_INPUT);
            }
            _skip_whitespace();

            if ((L'\"' == _get_current_char()) ||
                (L'\'' == _get_current_char()))
            {
                return _collect_string();
            }

            if (std::isdigit(_get_current_char()))
            {
                return _collect_number();
            }

            if ((std::isalpha(_get_current_char())) ||
                (L'_' == _get_current_char()))
            {
                return _collect_identifier();
            }

            if (Token::_s_is_symbol(_get_current_char()))
            {
                _skip_comments();
                if ((_is_start_of_comments()) ||
                    (!Token::_s_is_symbol(_get_current_char())))
                {
                    continue;
                }
                return _collect_symbol();
            }

            if (L'\n' == _get_current_char())
            {
                Location newline_location = m_location;
                _advance();
                return Token(newline_location, TokenType::NEW_LINE, L"");
            }

            throw KodException(KodStatus::KODSTATUS_LEXER_GET_NEXT_TOKEN_UNKNOWN_CHARARCTER, _get_current_char());
        }
    }

    Token Lexer::peek_token()
    {
        // Saving state
        Location old_location = m_location;
        uint32_t old_index = m_index;

        Token token = get_next_token();

        // Restoring old state
        m_location = old_location;
        m_index = old_index;

        return token;
    }

    void Lexer::_skip_whitespace()
    {
        while (
            (_can_advance()) &&
            (std::isspace(_get_current_char())) &&
            (_get_current_char() != '\n')
            ) 
        {
            _advance();
        }
    }

    void Kod::Lexer::_skip_comments()
    {
        // TODO
    }

    wchar_t Lexer::_get_current_char() const
    {
        if (!_can_advance())
        {
            throw KodException(KodStatus::KODSTATUS_LEXER_GET_CURRENT_CHAR_INDEX_OUT_OF_RANGE, m_index);
        }
        return m_content[m_index];
    }

    wchar_t Lexer::_peek_char() const
    {
        if (!_can_advance())
        {
            throw KodException(KodStatus::KODSTATUS_LEXER_PEEK_CHAR_INDEX_OUT_OF_RANGE, m_index);
        }
        return m_content[m_index + 1];
    }

    void Lexer::_advance()
    {
        if (!_can_advance()) return;

        if (_get_current_char() == '\n') {
            m_location.new_line();
        }
        else {
            m_location.add_column();
        }

        ++m_index;
    }

    bool Lexer::_can_advance() const
    {
        return (m_content.size() > m_index) &&
            (L'\0' != m_content[m_index]);
    }

    bool Kod::Lexer::_is_start_of_comments() const
    {
        if (!_can_advance()) return false;

        return (L'/' == _get_current_char()) &&
            ((L'/' == _peek_char()) || (L'*' == _peek_char()));
    }

    Token Lexer::_collect_string()
    {
        const bool is_single_quote = L'\'' == _get_current_char();
        const Location string_location = m_location;
        std::wstring string;

        // Eating first quote/s
        _advance();

        while (_can_advance()
            && (((L'\'' != _get_current_char()) && is_single_quote) ||
                ((L'"' != _get_current_char()) && !is_single_quote)))
        {
            if (L'\\' == _get_current_char())
            {
                _advance();
                if (!_can_advance())
                {
                    break;
                }

                // check what kind of escape
                switch (_get_current_char())
                {
                case L'b':  string.push_back(L'\x08'); break;
                case L'n':  string.push_back(L'\n'); break;
                case L't':  string.push_back(L'\t'); break;
                case L'\\': string.push_back(L'\\'); break;
                case L'r':  string.push_back(L'\r'); break;
                case L'\'': string.push_back(L'\''); break;
                case L'\"': string.push_back(L'\"'); break;
                default: break;// TODO: log a warning
                }
                _advance();
                continue;
            }
            string.push_back(_get_current_char());
            _advance();
        }

        // Check if end of string
        if ((L'\'' != _get_current_char() && is_single_quote) ||
            ((L'"' != _get_current_char() && !is_single_quote)))
        {
            throw KodException(KodStatus::KODSTATUS_LEXER_COLLECT_STRING_UNTERMINATED_STRING, m_index);
        }

        // Eat the other quote/s
        _advance();

        return Token(string_location, TokenType::STRING, string);
    }

    Token Lexer::_collect_number()
    {
        // TODO
        return Token();
    }

    Token Lexer::_collect_identifier()
    {
        std::wstring identifier;
        TokenType ttype = TokenType::ID;
        Location id_location = m_location;

        while ((_can_advance()) && 
            (std::isalnum(_get_current_char()) || L'_' == _get_current_char()))
        {
            identifier.push_back(_get_current_char());
            _advance();
        }

        KeywordType ktype = Token::s_keyword_to_type(identifier);

        if (KeywordType::UNKNOWN != ktype)
        {
            ttype = TokenType::KEYWORD;
        }

        auto tok = Token(id_location, ttype, identifier, ktype);
        return tok;
    }

    Token Lexer::_collect_symbol()
    {
        Location symbol_location = m_location;
        std::wstring symbol;

        symbol.push_back(_get_current_char());
        _advance();

        TokenType ttype = Token::s_symbol_to_type(symbol);

        // If it's a double (or more) character symbol
        if ((_can_advance()) && 
            (Token::_s_is_symbol(_get_current_char())))
        {
            std::wstring new_symbol = symbol + _get_current_char();

            TokenType second_type = Token::s_symbol_to_type(new_symbol);

            if (TokenType::UNKNOWN != second_type)
            {
                ttype = second_type;
                symbol = new_symbol;
                _advance();
            }
        }

        return Token(symbol_location, ttype, symbol);
    }
}
