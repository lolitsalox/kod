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

        virtual std::wstring to_string(uint32_t level = 0) const = 0;

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
    
    class AstNodeCompound : public AstNode
    {
    public:
        explicit AstNodeCompound(std::vector<AstNodeUPtr> nodes = {}) :
            m_nodes(std::move(nodes)) {}
        ~AstNodeCompound() override = default;
        AstNodeCompound(const AstNodeCompound&) = delete;
        AstNodeCompound& operator=(const AstNodeCompound&) = delete;
        AstNodeCompound(AstNodeCompound&&) = delete;
        AstNodeCompound& operator=(AstNodeCompound&&) = delete;

        void push_back(AstNodeUPtr node)
        {
            m_nodes.push_back(std::move(node));
        }

        std::wstring to_string(uint32_t level = 0) const override 
        {
            std::wstringstream result;
            result << std::wstring(level * 4, L' ') << L"{\n";
            for (auto&& node : m_nodes)
            {
                result << node->to_string(level + 1) + L"\n";
            }
            result << std::wstring(level * 4, L' ') << L"}";
            return result.str();
        }

    private:
        std::vector<AstNodeUPtr> m_nodes;
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

        virtual std::wstring to_string(uint32_t) const override 
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

        virtual std::wstring to_string(uint32_t) const override { return m_identifier; }

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

        virtual std::wstring to_string(uint32_t) const override { return m_number; }

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

        virtual std::wstring to_string(uint32_t level = 0) const override 
        {
            std::wstringstream ss;
            ss << std::wstring(level * 4, L' ');
            ss << L"(" << m_lhs->to_string(level + 1) << L" " <<
               m_operator.get_value() << L" " <<
               m_rhs->to_string(level + 1) << L")";
            return ss.str();
        }

    private:
        const Token m_operator;
        const AstNodeUPtr m_lhs;
        const AstNodeUPtr m_rhs;
    };
}
