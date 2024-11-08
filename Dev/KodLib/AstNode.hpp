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
        explicit AstNodeString(const std::wstring& string) : 
            m_string(string) {}
        virtual ~AstNodeString() = default;
        AstNodeString(const AstNodeString&) = delete;
        AstNodeString& operator=(const AstNodeString&) = delete;
        AstNodeString(AstNodeString&&) = delete;
        AstNodeString& operator=(AstNodeString&&) = delete;

        virtual std::wstring to_string() const override 
        { 
            std::wostringstream ss;
            ss << std::quoted(m_string); 
            return ss.str();
        }

    private:
        const std::wstring m_string;
    };

    class AstNodeIdentifier : public AstNode
    {
    public:
        explicit AstNodeIdentifier(const std::wstring& identifier) :
            m_identifier(identifier) {}
        virtual ~AstNodeIdentifier() = default;
        AstNodeIdentifier(const AstNodeIdentifier&) = delete;
        AstNodeIdentifier& operator=(const AstNodeIdentifier&) = delete;
        AstNodeIdentifier(AstNodeIdentifier&&) = delete;
        AstNodeIdentifier& operator=(AstNodeIdentifier&&) = delete;

        virtual std::wstring to_string() const override { return m_identifier; }

    private:
        const std::wstring m_identifier;
    };

    class AstNodeNumber : public AstNode
    {
    public:
        explicit AstNodeNumber(const std::wstring& number) :
            m_number(number) {}
        virtual ~AstNodeNumber() = default;
        AstNodeNumber(const AstNodeNumber&) = delete;
        AstNodeNumber& operator=(const AstNodeNumber&) = delete;
        AstNodeNumber(AstNodeNumber&&) = delete;
        AstNodeNumber& operator=(AstNodeNumber&&) = delete;

        virtual std::wstring to_string() const override { return m_number; }

    private:
        const std::wstring m_number;
    };

    class AstNodeBinaryOp : public AstNode
    {
    public:
        AstNodeBinaryOp(const Token op, AstNodeUPtr lhs, AstNodeUPtr rhs) : 
            m_operator(op), m_lhs(std::move(lhs)), m_rhs(std::move(rhs)) {}
        virtual ~AstNodeBinaryOp() = default;
        AstNodeBinaryOp(const AstNodeBinaryOp&) = delete;
        AstNodeBinaryOp& operator=(const AstNodeBinaryOp&) = delete;
        AstNodeBinaryOp(AstNodeBinaryOp&&) = delete;
        AstNodeBinaryOp& operator=(AstNodeBinaryOp&&) = delete;

        virtual std::wstring to_string() const override 
        { 
            return L"(" + m_lhs->to_string() + L" " + m_operator.get_value() + L" " + m_rhs->to_string() + L")"; 
        }

    private:
        const Token m_operator;
        const AstNodeUPtr m_lhs;
        const AstNodeUPtr m_rhs;
    };
}
