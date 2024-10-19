#include "AstNode.hpp"
#include "Parser.hpp"

namespace Kod
{
    Parser::Parser(LexerUPtr lexer) :
        m_ast_node_factory(AstNodeFactory::s_get_instance()),
        m_lexer(std::move(lexer)),
        m_token(m_lexer->get_next_token())
    {}
    
    AstNodeUPtr Parser::parse()
    {
        return _assignment();
    }

    void Parser::_next_token()
    {
        m_token = m_lexer->get_next_token();
    }

    void Parser::_eat_token(const TokenType type)
    {
        if (type != m_token.get_type())
        {
            // TODO: Log error
            throw std::exception("bad token");
        }
        _next_token();
    }

    bool Parser::_optional_eat_token(const TokenType type)
    {
        if (type != m_token.get_type())
        {
            return false;
        }
        _next_token();
        return true;
    }

    AstNodeUPtr Parser::_assignment()
    {
        AstNodeUPtr lhs = _factor();
        
        if (!_optional_eat_token(TokenType::EQUALS))
        {
            return lhs;
        }

        AstNodeUPtr rhs = _assignment();
        return std::make_unique<AstNodeAssignment>(std::move(lhs), std::move(rhs));
    }

    AstNodeUPtr Parser::_factor()
    {
        AstNodeUPtr node = m_ast_node_factory->create(m_token);
        _next_token();
        return node;
    }
}
