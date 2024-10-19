#pragma once

#include "Lexer.hpp"
#include "AstNode.hpp"

namespace Kod
{
    class Parser
    {
    public:
        Parser(LexerUPtr lexer);
        virtual ~Parser() = default;
        Parser(const Parser&) = delete;
        Parser& operator=(const Parser&) = delete;
        Parser(Parser&&) = delete;
        Parser& operator=(Parser&&) = delete;

        AstNodeUPtr parse();

    private:
        void _next_token();
        void _eat_token(const TokenType type);
        bool _optional_eat_token(const TokenType type);
        AstNodeUPtr _assignment();
        AstNodeUPtr _factor();

    private:
        const AstNodeFactoryPtr m_ast_node_factory;
        const LexerUPtr m_lexer;
        Token m_token;
    };
}
