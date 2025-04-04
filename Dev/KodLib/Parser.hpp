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
        AstNodeUPtr _plus_minus();
        AstNodeUPtr _mul_div_mod();
        AstNodeUPtr _pow();
        AstNodeUPtr _factor();

        template<typename F, typename... TokenTypes>
        AstNodeUPtr _binary(F&& next_function, TokenTypes... token_types);

    private:
        const AstNodeFactoryPtr m_ast_node_factory;
        const LexerUPtr m_lexer;
        Token m_token;
    };

    template<typename F, typename ...TokenTypes>
    inline AstNodeUPtr Parser::_binary(F&& next_function, TokenTypes ...token_types)
    {
        AstNodeUPtr lhs = next_function();

        const Token op = m_token;
        while ((_optional_eat_token(token_types) || ...))
        {
            AstNodeUPtr rhs = next_function();
            lhs = std::make_unique<AstNodeBinaryOp>(op, std::move(lhs), std::move(rhs));
        }
        return lhs;
    }
}
