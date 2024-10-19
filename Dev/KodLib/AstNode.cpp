#include "Token.hpp"
#include "AstNode.hpp"

namespace Kod
{
    std::wostream& operator<<(std::wostream& os, const AstNode& node)
    {
        os << node.to_string();
        return os;
    }

    AstNodeFactoryPtr AstNodeFactory::g_instance = nullptr;

    AstNodeFactoryPtr AstNodeFactory::s_get_instance()
    {
        if (nullptr == g_instance)
        {
            g_instance = std::shared_ptr<AstNodeFactory>(new AstNodeFactory());
        }
        return g_instance;
    }

    AstNodeUPtr AstNodeFactory::create(const Token token)
    {
        return std::make_unique<AstNodeString>(token.get_value());
    }
}
