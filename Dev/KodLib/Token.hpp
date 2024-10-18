#pragma once

#include "Common.hpp"

namespace Kod
{
    enum class TokenType : uint32_t
    {
        PLUS,               //  +
        MINUS,              //  -
        DIV,                //  /
        MUL,                //  *
        MOD,                //  %
        POW,                //  **

        PLUS_EQ,            //  +=
        MINUS_EQ,           //  -=
        DIV_EQ,             //  /=
        MUL_EQ,             //  *=
        MOD_EQ,             //  %=

        AND,                //  &
        OR,                 //  |
        HAT,                //  ^
        SHL,                //  <<
        SHR,                //  >>
        NOT,                //  ~

        BOOL_NOT,           //  !
        BOOL_EQ,            //  ==
        BOOL_NE,            //  !=
        BOOL_LT,            //  <
        BOOL_GT,            //  >
        BOOL_LTE,           //  <=
        BOOL_GTE,           //  >=
        BOOL_AND,           //  &&
        BOOL_OR,            //  ||
        ID,                 //  main x y foo
        KEYWORD,            //  NOT USED
        SIZEOF,             //  sizeof

        CHAR,               //  'a'
        STRING,             //  "Hello world"
        INT,                //  5 6 456
        FLOAT,              //  6.9 7893.6   

        LPAREN,             //  (   
        RPAREN,             //  )              
        LBRACKET,           //  [              
        RBRACKET,           //  ]              
        LBRACE,             //  {          
        RBRACE,             //  }

        EQUALS,             //  =   
        COMMA,              //  ,  
        DOT,                //  .  
        COLON,              //  :  
        NAMESPACE,          //  ::  
        SEMI,               //  ;   
        QUESTION,           //  ?   
        AT,                 //  @
        HASH,               //  #
        LINE_COMMENT,       // //
        MULTILINE_COMMENT_START,     // /*
        MULTILINE_COMMENT_END,       // */
        POINTER,            //  ->
        ARROW,              //  =>
        BACKSLASH,          // 

        NEW_LINE,                 //  New line
        END_OF_INPUT,             //  The end of the input
    };

    enum class KeywordType {
        NULL_K,
        TRUE,
        FALSE,
        IF,
        ELSE,
        WHILE,
        FOR,
        RETURN,
        IMPORT,
        AS,
        FROM,
        BREAK,
        CONTINUE,
    };

    class Location
    {
    public:
        Location(const std::wstring& file_path = L"", uint32_t line = 1, uint32_t column = 1);
        virtual ~Location() = default;
        Location(const Location&) = default;
        Location& operator=(const Location&) = default;
        Location(Location&&) = default;
        Location& operator=(Location&&) = default;

        [[nodiscard]] inline uint32_t get_line() const { return m_line; }
        [[nodiscard]] inline uint32_t get_column() const { return m_column; }

        inline void new_line() { ++m_line; m_column = 1; }
        inline void add_column() { ++m_column; }

    private:
        friend std::wostream& operator<<(std::wostream& os, const Location& location);

    private:
        std::wstring m_file_path;
        uint32_t m_line;
        uint32_t m_column;
    };

    class Token
    {
    public:
        explicit Token(const Location location = {}, const TokenType type = TokenType::END_OF_INPUT, const std::wstring& value = L"");
        virtual ~Token() = default;
        Token(const Token&) = default;
        Token& operator=(const Token&) = default;
        Token(Token&&) = default;
        Token& operator=(Token&&) = default;

        [[nodiscard]] TokenType get_type() const { return m_type; }
        [[nodiscard]] std::wstring get_value() const { return m_value; }
        [[nodiscard]] Location get_location() const { return m_location; }

        [[nodiscard]] bool is(const TokenType type) const { return m_type == type; }

    private:
        friend std::wostream& operator<<(std::wostream& os, const Token& token);

    private:
        TokenType m_type;
        Location m_location;
        std::wstring m_value;
    };
}
