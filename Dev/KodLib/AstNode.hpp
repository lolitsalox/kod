#pragma once

#include "Token.hpp"

namespace Kod
{
    class AstNode
    {
    public:
        AstNode() = default;
        virtual ~AstNode() = default;
        AstNode(const AstNode&) = delete;
        AstNode& operator=(const AstNode&) = delete;
        AstNode(AstNode&&) = delete;
        AstNode& operator=(AstNode&&) = delete;

        virtual std::wstring to_string() const = 0;

    private:
        friend std::wostream& operator<<(std::wostream& os, const AstNode& node);
    };
    using AstNodeUPtr = std::unique_ptr<AstNode>;

    class AstNodeFactory;
    using AstNodeFactoryPtr = std::shared_ptr<AstNodeFactory>;

    class AstNodeFactory
    {
    public:
        virtual ~AstNodeFactory() = default;
        AstNodeFactory(const AstNodeFactory&) = delete;
        AstNodeFactory& operator=(const AstNodeFactory&) = delete;
        AstNodeFactory(AstNodeFactory&&) = delete;
        AstNodeFactory& operator=(AstNodeFactory&&) = delete;

        AstNodeUPtr create(const Token token);
        static AstNodeFactoryPtr s_get_instance();

    private:
        // Singleton
        AstNodeFactory() = default;

    private:
        static AstNodeFactoryPtr g_instance;
    };

    class AstNodeString : public AstNode
    {
    public:
        AstNodeString(const std::wstring& string) : m_string(string) {}
        virtual ~AstNodeString() = default;
        AstNodeString(const AstNodeString&) = delete;
        AstNodeString& operator=(const AstNodeString&) = delete;
        AstNodeString(AstNodeString&&) = delete;
        AstNodeString& operator=(AstNodeString&&) = delete;

        virtual std::wstring to_string() const override { return m_string; }

    private:
        const std::wstring m_string;
    };

    class AstNodeAssignment : public AstNode
    {
    public:
        AstNodeAssignment(AstNodeUPtr lhs, AstNodeUPtr rhs) : m_lhs(std::move(lhs)), m_rhs(std::move(rhs)) {}
        virtual ~AstNodeAssignment() = default;
        AstNodeAssignment(const AstNodeAssignment&) = delete;
        AstNodeAssignment& operator=(const AstNodeAssignment&) = delete;
        AstNodeAssignment(AstNodeAssignment&&) = delete;
        AstNodeAssignment& operator=(AstNodeAssignment&&) = delete;

        virtual std::wstring to_string() const override { return m_lhs->to_string() + L" = " + m_rhs->to_string(); }

    private:
        const AstNodeUPtr m_lhs;
        const AstNodeUPtr m_rhs;
    };
}
